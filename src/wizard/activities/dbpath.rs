use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use super::super::wizard_activity::WizardActivityTrait;
use crate::theme::ITEM_HEIGHT;
use fltk::button::{Button, CheckButton};
use fltk::dialog::NativeFileChooser;
use fltk::enums::{Align, CallbackTrigger};
use fltk::frame::Frame;
use fltk::group::Flex;
use fltk::image::PngImage;
use fltk::input::FileInput;
use fltk::prelude::*;

static ICON_DATA: &'static [u8] = include_bytes!("images/dbpath_noBG.png");

pub struct DBPathActivity {
    db_path: Rc<RefCell<Option<PathBuf>>>,
}

impl WizardActivityTrait for DBPathActivity {
    fn start_activity(&mut self) {
        let mut main_layout = Flex::default().column();
        let mut layout = Flex::default().row();
        Frame::default()
            .with_align(Align::Inside | Align::Left)
            .with_label(
                "\
        Optionally choose an alternative location for Scheduler's database\n\
        \n\
        If you are not planning to run scheduler on this machine - just skip\n\
        this step\n\
        \n\
        Scheduler's database is the file where the whole state of the\n\
        processing is stored.\
        ",
            );
        let mut icon = Frame::default();
        icon.set_image(Some(PngImage::from_data(ICON_DATA).unwrap()));
        layout.fixed(&icon, 128);
        layout.end();
        main_layout.fixed(&layout, 160);

        Frame::default()
            .with_align(Align::Inside | Align::Left)
            .with_label(
                "\
        By default it is stored in your user directory, and it's a fine location, but you may choose\n\
        a different place.\n\
        \n\
        Database should preferably be located on a fast device (like SSD).\n\
        Database should NOT be network-accessible, it is DISCOURAGED to store\n\
        the database file on a network filesystem.\n\
        \n\
        Considering all this, you may choose to provide an alternative directory where\n\
        the database will be stored below.
        ",
            );

        let mut layout = Flex::default().row();
        let mut do_dbpath = CheckButton::default().with_label("override");
        layout.fixed(&do_dbpath, 80);
        let mut dbpath = FileInput::default();
        let mut browse_dbpath = Button::default().with_label("browse");
        layout.fixed(&browse_dbpath, 64);
        layout.end();
        main_layout.fixed(&layout, ITEM_HEIGHT);

        main_layout.end();

        // some init
        if let Some(path) = self.db_path.borrow().as_ref() {
            dbpath.set_value(&path.to_string_lossy());
            do_dbpath.set_value(true);
        } else {
            dbpath.deactivate();
            browse_dbpath.deactivate();
        }

        // callbacks
        do_dbpath.set_callback({
            let mut dbpath = dbpath.clone();
            let mut browse_dbpath = browse_dbpath.clone();
            let override_db_path = self.db_path.clone();
            move |w| {
                if w.value() {
                    dbpath.activate();
                    browse_dbpath.activate();
                    *override_db_path.borrow_mut() = Some(PathBuf::from(dbpath.value()));
                } else {
                    dbpath.deactivate();
                    browse_dbpath.deactivate();
                    *override_db_path.borrow_mut() = None;
                }
            }
        });

        browse_dbpath.set_callback({
            let mut dbpath = dbpath.clone();
            let override_db_path = self.db_path.clone();
            move |_| {
                let mut dialog =
                    NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseDir);
                dialog.show();
                let input_path = dialog.filename();
                let input_str = &input_path.to_string_lossy();
                dbpath.set_value(input_str);
                *override_db_path.borrow_mut() = Some(input_path);
            }
        });

        dbpath.set_trigger(CallbackTrigger::Changed);
        dbpath.set_callback({
            let override_db_path = self.db_path.clone();
            move |w| {
                *override_db_path.borrow_mut() = Some(PathBuf::from(w.value()));
            }
        });
    }

    fn contents_size(&self) -> (i32, i32) {
        (640, 500)
    }

    fn validate(&self) -> Result<(), &str> {
        match *self.db_path.borrow() {
            None => return Ok(()),
            Some(ref path) => {
                if !path.exists() {
                    return Err("given path does not exist");
                }
                return Ok(());
            }
        }
    }
}

impl DBPathActivity {
    pub fn new() -> Self {
        DBPathActivity {
            db_path: Rc::new(RefCell::new(None)),
        }
    }

    pub fn from_path(path: &Path) -> Self {
        DBPathActivity {
            db_path: Rc::new(RefCell::new(Some(path.to_owned()))),
        }
    }

    pub fn selected_path(&self) -> Option<PathBuf> {
        if let Some(path) = self.db_path.borrow().as_ref() {
            Some(path.to_owned())
        } else {
            None
        }
    }
}
