use std::cell::Cell;
use std::rc::Rc;

use super::super::wizard_activity::WizardActivityTrait;
use fltk::button::CheckButton;
use fltk::enums::Align;
use fltk::group::Flex;
use fltk::image::PngImage;
use fltk::{frame::Frame, prelude::*};

static ICON_DATA: &'static [u8] = include_bytes!("images/think_noBG.png");

pub struct DCCTypesActivity {
    do_blender: Rc<Cell<bool>>,
    do_houdini: Rc<Cell<bool>>,
    do_redshift: Rc<Cell<bool>>,
}

impl WizardActivityTrait for DCCTypesActivity {
    fn start_activity(&mut self) {
        let mut layout = Flex::default().row();
        let mut icon = Frame::default();
        icon.set_image(Some(PngImage::from_data(ICON_DATA).unwrap()));
        layout.fixed(&icon, 128);
        Frame::default()
            .with_align(Align::Inside | Align::Left)
            .with_label(
                "\
        Being a wizard, I know how to set up some DCCs and tools for Lifeblood\n\
        \n\
        Sellect which ones you would like to set up.\
        ",
            );
        layout.end();

        let mut do_houdini_checkbox = CheckButton::default().with_label("SideFX Houdini");
        do_houdini_checkbox.set_callback({
            let do_houdini = self.do_houdini.clone();
            move |w| {
                do_houdini.replace(w.value());
            }
        });
        let mut do_redshift_checkbox = CheckButton::default().with_label("Redshift");
        do_redshift_checkbox.set_callback({
            let do_redshift = self.do_redshift.clone();
            move |w| {
                do_redshift.replace(w.value());
            }
        });
        let mut do_blender_checkbox = CheckButton::default().with_label("Blender");
        do_blender_checkbox.set_callback({
            let do_blender = self.do_blender.clone();
            move |w| {
                do_blender.replace(w.value());
            }
        });

        // init
        do_houdini_checkbox.set_value(self.do_houdini.get());
        do_redshift_checkbox.set_value(self.do_redshift.get());
        do_blender_checkbox.set_value(self.do_blender.get());
    }

    fn contents_size(&self) -> (i32, i32) {
        (650, 400)
    }

    fn validate(&self) -> Result<(), &str> {
        Ok(())
    }
}

impl DCCTypesActivity {
    pub fn new(do_blender: bool, do_houdini: bool, do_redshift: bool) -> Self {
        DCCTypesActivity {
            do_blender: Rc::new(Cell::new(do_blender)),
            do_houdini: Rc::new(Cell::new(do_houdini)),
            do_redshift: Rc::new(Cell::new(do_redshift)),
        }
    }

    pub fn selected_dccs(&self) -> (bool, bool, bool) {
        (self.do_blender.get(), self.do_houdini.get(), self.do_redshift.get())
    }
}
