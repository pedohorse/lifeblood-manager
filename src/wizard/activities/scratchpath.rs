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

static ICON_DATA: &'static [u8] = include_bytes!("images/browse_noBG.png");

pub struct ScratchLocationPathActivity {
    scratch_path: Rc<RefCell<Option<PathBuf>>>,
}

impl WizardActivityTrait for ScratchLocationPathActivity {
    fn start_activity(&mut self) {
        let mut main_layout = Flex::default().column();
        let mut layout = Flex::default().row();
        Frame::default()
            .with_align(Align::Inside | Align::Left)
            .with_label(
                "\
        If you have multiple workers over the network - you must specify\n\
        a shared network location where workers can exchange with files\n\
        \n\
        For example, temporary ifd, ass, rs or usd files will be created\n\
        in that location by one worker to be rendered by another\n\
        before being cleaned up\n\
        \n\
        If you are not planning to use more than a single computer\n\
        with Lifeblood - you can skip this step\n\
        \n\
        You can also later change this location in scheduler's config files.\n\
        ",
            );
        let mut icon = Frame::default();
        icon.set_image(Some(PngImage::from_data(ICON_DATA).unwrap()));
        layout.fixed(&icon, 128);
        layout.end();
        main_layout.fixed(&layout, 160);

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
        if let Some(path) = self.scratch_path.borrow().as_ref() {
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
            let override_db_path = self.scratch_path.clone();
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
            let override_db_path = self.scratch_path.clone();
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
            let override_db_path = self.scratch_path.clone();
            move |w| {
                *override_db_path.borrow_mut() = Some(PathBuf::from(w.value()));
            }
        });
    }

    fn contents_size(&self) -> (i32, i32) {
        (640, 500)
    }

    fn validate(&self) -> Result<(), &str> {
        match *self.scratch_path.borrow() {
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

impl ScratchLocationPathActivity {
    pub fn new() -> Self {
        ScratchLocationPathActivity {
            scratch_path: Rc::new(RefCell::new(None)),
        }
    }

    pub fn from_path(path: &Path) -> Self {
        ScratchLocationPathActivity {
            scratch_path: Rc::new(RefCell::new(Some(path.to_owned()))),
        }
    }

    pub fn selected_path(&self) -> Option<PathBuf> {
        if let Some(path) = self.scratch_path.borrow().as_ref() {
            Some(path.to_owned())
        } else {
            None
        }
    }
}
