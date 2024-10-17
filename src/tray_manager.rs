use fltk::image::PngImage;
use fltk::prelude::ImageExt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{mpsc, Arc, Mutex};
use tray_item::{IconSource, TrayItem};

#[cfg(not(windows))]
// tray_item expects image to be ARGB, and it is easier to pre-gen such image
// rather than swap pixels in runtime
const TRAY_ICON: &'static [u8] = include_bytes!("../icon_argb.png");

#[derive(Debug)]
enum TrayMessage {
    WidgetMessage((usize, u32)),
}

pub struct TrayManager {
    tray: Rc<RefCell<TrayItem>>,
    tray_command_sender: mpsc::Sender<TrayMessage>,
    tray_command_receiver: mpsc::Receiver<TrayMessage>,
    next_tray_id: usize,
    tray_callbacks: HashMap<usize, Box<dyn FnMut(&mut TrayItemHandle) -> ()>>,
}

#[derive(Clone)]
pub struct TrayItemHandle {
    tray: Rc<RefCell<TrayItem>>,
    tray_id: u32,
}

impl TrayManager {
    pub fn new(title: &str) -> Result<TrayManager, &str> {
        let (tx, rx) = mpsc::channel();

        #[cfg(not(windows))]
        let icon = {
            let img = PngImage::from_data(TRAY_ICON).unwrap();
            let rgb_data = img.to_rgb_data();

            IconSource::Data {
                height: img.height(),
                width: img.width(),
                data: rgb_data,
            }
        };
        #[cfg(windows)]
        let icon = IconSource::Resource("tray_icon");
        
        let tray_item = match TrayItem::new(title, icon) {
            Ok(x) => x,
            Err(_) => {
                return Err("failed to create tray item");
            }
        };
        Ok(TrayManager {
            tray: Rc::new(RefCell::new(tray_item)),
            tray_command_sender: tx,
            tray_command_receiver: rx,
            next_tray_id: 0,
            tray_callbacks: HashMap::new(),
        })
    }

    pub fn add_tray_item<F>(&mut self, label: &str, callback: F) -> Result<TrayItemHandle, ()>
    where
        F: FnMut(&mut TrayItemHandle) -> () + 'static,
    {
        let tray = &mut *self.tray.borrow_mut();
        let item_id = Arc::new(Mutex::new(0));
        if let Ok(id) = tray.inner_mut().add_menu_item_with_id(&label, {
            let tx = self.tray_command_sender.clone();
            let item_id = item_id.clone();
            let message_type_id = self.next_tray_id;
            move || {
                tx.send(TrayMessage::WidgetMessage((
                    message_type_id,
                    *item_id.lock().unwrap(),
                )))
                .unwrap_or_else(|_| {
                    println!("failed to communicate from tray item");
                });
            }
        }) {
            *item_id.lock().unwrap() = id;
            self.tray_callbacks
                .insert(self.next_tray_id, Box::new(callback));
            self.next_tray_id += 1;
            Ok(TrayItemHandle {
                tray: self.tray.clone(),
                tray_id: id,
            })
        } else {
            eprintln!("failed to add tray item {}", label);
            Err(())
        }
    }

    ///
    /// returns false if no further processing is needed
    pub fn process_tray_messages(&mut self) -> bool {
        match self.tray_command_receiver.try_recv() {
            Ok(message) => {
                println!("i have received {:?}", message);
                match message {
                    TrayMessage::WidgetMessage((message_type_id, tray_item_id)) => {
                        if let Some(callback) = self.tray_callbacks.get_mut(&message_type_id) {
                            callback(&mut TrayItemHandle {
                                tray: self.tray.clone(),
                                tray_id: tray_item_id,
                            });
                        } else {
                            eprintln!("tray message type {} has no callback!", message_type_id);
                        }
                    }
                }
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => return false, // close control channel
        };
        true
    }
}

impl TrayItemHandle {
    pub fn change_label(&mut self, label: &str) -> Result<(), ()> {
        match self
            .tray
            .borrow_mut()
            .inner_mut()
            .set_menu_item_label(label, self.tray_id)
        {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }
}
