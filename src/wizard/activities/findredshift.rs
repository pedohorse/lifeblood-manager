use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use super::super::wizard_activity::WizardActivityTrait;
use crate::theme::ITEM_HEIGHT;
use fltk::button::Button;
use fltk::dialog::NativeFileChooser;
use fltk::enums::Align;
use fltk::group::Flex;
use fltk::image::PngImage;
use fltk::input::{FileInput, IntInput};
use fltk::misc::Spinner;
use fltk::{frame::Frame, prelude::*};

static ICON_DATA: &'static [u8] = include_bytes!("images/browse_noBG.png");

pub struct FindRedshiftActivity {
    widgets: Option<
        Rc<
            RefCell<
                Vec<(
                    Flex,
                    FileInput,
                    IntInput,
                    IntInput,
                    IntInput,
                )>,
            >,
        >,
    >,
    init_data: Vec<(PathBuf, (u32, u32, u32))>,
}

impl WizardActivityTrait for FindRedshiftActivity {
    fn start_activity(&mut self) {
        let mut main_layout = Flex::default().column();
        let mut layout = Flex::default().row();
        let mut icon = Frame::default();
        icon.set_image(Some(PngImage::from_data(ICON_DATA).unwrap()));
        layout.fixed(&icon, 144);
        Frame::default()
            .with_align(Align::Inside | Align::Left)
            .with_label(
                "\
        TBD
        ",
            );
        layout.end();
        main_layout.fixed(&layout, 128);

        const MAX_VER_NUM: usize = 8;
        
        let mut layout = Flex::default().row();
        let version_count_label = Frame::default().with_label("number of versions");
        layout.fixed(&version_count_label, 140);
        let mut version_number_spinner = Spinner::default();
        layout.fixed(&version_number_spinner, 48);
        version_number_spinner.set_step(1.0);
        version_number_spinner.set_minimum(1 as f64);
        version_number_spinner.set_maximum(MAX_VER_NUM as f64);
        layout.end();
        main_layout.fixed(&layout, ITEM_HEIGHT);

        let mut user_inputs = Vec::new();
        for i in 0..MAX_VER_NUM {
            let mut row_layout = Flex::default().row();
            let bin_label = Frame::default().with_label("path to bin");
            row_layout.fixed(&bin_label, 72);
            let bin_path = FileInput::default();
            let mut browse_btn = Button::default().with_label("browse");
            let ver_label = Frame::default().with_label("version:");
            let ver_maj = IntInput::default();
            let ver_min = IntInput::default();
            let ver_patch = IntInput::default();
            row_layout.fixed(&browse_btn, 80);
            row_layout.fixed(&ver_label, 56);
            row_layout.fixed(&ver_maj, 64);
            row_layout.fixed(&ver_min, 32);
            row_layout.fixed(&ver_patch, 48);
            row_layout.end();

            main_layout.fixed(&row_layout, ITEM_HEIGHT);
            if i > 0 {
                row_layout.hide();
            }

            // browse callback
            browse_btn.set_callback({
                let mut bin_path = bin_path.clone();
                move |_| {
                    let mut dialog =
                        NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseDir);
                    dialog.show();
                    let input_path = dialog.filename();
                    let input_str = &input_path.to_string_lossy();
                    bin_path.set_value(input_str);
                }
            });

            user_inputs.push((
                row_layout, bin_path, ver_maj, ver_min, ver_patch
            ));
        }
        main_layout.end();

        // init
        version_number_spinner.set_value(self.init_data.len().max(1) as f64);
        for (i, ver_data) in self.init_data.iter().enumerate() {
            if i > MAX_VER_NUM {
                break;
            }
            let input_data = &mut user_inputs[i];
            input_data.0.show();
            input_data.1.set_value(&ver_data.0.to_string_lossy());
            input_data.2.set_value(&ver_data.1 .0.to_string());
            input_data.3.set_value(&ver_data.1 .1.to_string());
            input_data.4.set_value(&ver_data.1 .2.to_string());
        }

        self.widgets = Some(Rc::new(RefCell::new(user_inputs)));

        // callbacks

        version_number_spinner.set_callback({
            let widgets = self.widgets.as_ref().unwrap().clone();
            move |w| {
                let number_of_versions = w.value() as usize;
                for i in 0..number_of_versions {
                    widgets.borrow_mut()[i].0.show();
                }
                for i in number_of_versions..MAX_VER_NUM {
                    widgets.borrow_mut()[i].0.hide();
                }
                main_layout.layout();
            }
        })
    }

    fn contents_size(&self) -> (i32, i32) {
        (800, 500)
    }

    fn validate(&self) -> Result<(), &str> {
        // just checking paths to have houdini bin in it
        if let Some(ref widget_tuples) = self.widgets {
            for widget_tuple in widget_tuples.borrow().iter() {
                if !widget_tuple.0.visible() {
                    break;
                }
                // first check bin path
                let bin_path = PathBuf::from(widget_tuple.1.value());
                if !bin_path.is_absolute() {
                    return Err("provided path to redshift bin is not an absolute path");
                }
                if !bin_path.exists() {
                    return Err("provided path to redshift bin does not exist");
                }
                if !bin_path
                    .join(if cfg!(windows) {
                        "redshiftCmdLine.exe"
                    } else {
                        "redshiftCmdLine"
                    })
                    .exists()
                {
                    return Err("provided path does not contain redshiftCmdLine executable");
                }
                // validate version numbers
                macro_rules! version_check {
                    ($ver:expr) => {
                        if let Err(_) = u32::from_str_radix(&$ver, 10) {
                            return Err("Redshift version components must be set");
                        }
                    };
                }
                version_check!(widget_tuple.2.value());
                version_check!(widget_tuple.3.value());
                version_check!(widget_tuple.4.value());
            }
        }
        Ok(())
    }
}

impl FindRedshiftActivity {
    pub fn new(init_data: Vec<(PathBuf, (u32, u32, u32))>) -> Self {
        FindRedshiftActivity {
            widgets: None,
            init_data: init_data,
        }
    }

    pub fn selected_versions(&self) -> Option<Vec<(PathBuf, (u32, u32, u32))>> {
        if let Some(ref widgets_ref) = self.widgets {
            let mut ret = Vec::new();
            for widgets in widgets_ref.borrow().iter() {
                if !widgets.0.visible() {
                    break;
                }
                ret.push((
                    PathBuf::from(widgets.1.value()),
                    (
                        u32::from_str_radix(&widgets.2.value(), 10).unwrap_or(0),
                        u32::from_str_radix(&widgets.3.value(), 10).unwrap_or(0),
                        u32::from_str_radix(&widgets.4.value(), 10).unwrap_or(0),
                    ),
                ));
            }
            Some(ret)
        } else {
            None
        }
    }
}
