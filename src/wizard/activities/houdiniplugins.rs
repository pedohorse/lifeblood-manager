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
use fltk::input::FileInput;
use fltk::misc::Spinner;
use fltk::{frame::Frame, prelude::*};

static ICON_DATA: &'static [u8] = include_bytes!("images/browse_noBG.png");

pub struct HoudiniToolsActivity {
    init_hfs_dirs: Vec<PathBuf>,
    widgets: Option<Rc<RefCell<Vec<(Flex, FileInput)>>>>,
    count_widget: Option<Spinner>,
}

impl HoudiniToolsActivity {
    pub fn new(target_dirs: Vec<PathBuf>) -> HoudiniToolsActivity {
        HoudiniToolsActivity {
            init_hfs_dirs: target_dirs,
            widgets: None,
            count_widget: None,
        }
    }

    pub fn get_tools_install_locations(&self) -> Option<Vec<PathBuf>> {
        if let (Some(widgets), Some(counter)) = (&self.widgets, &self.count_widget) {
            let widgets_borrowed = widgets.borrow();
            let paths_count = counter.value() as usize;
            let mut ret = Vec::with_capacity(widgets_borrowed.len());
            for (i, file_input) in widgets_borrowed.iter().enumerate() {
                if i >= paths_count {
                    continue;
                }
                ret.push(PathBuf::from(file_input.1.value()));
            }

            Some(ret)
        } else {
            None
        }
    }
}

impl WizardActivityTrait for HoudiniToolsActivity {
    fn start_activity(&mut self) {
        let mut main_layout = Flex::default().column();
        let mut layout = Flex::default().row();
        Frame::default()
            .with_align(Align::Inside | Align::Left)
            .with_label(
                "\
        Houdini will need the submitter node to be able to submit tasks to Lifeblood.\n\
        The most common and simple place where to put it is Houdini's user directory.\n\
        \n\
        You can provide any path(s) which your Houdini is set up to load.\n\
        (via HOUDINI_PATH, HSITE or HOUDINI_USER_PREF_DIR)\n\
        \n\
        Alternatively, you can provide a path to an empty directory here to\n\
        be able to manually review and copy Lifeblood tools files by hand\n\
        after the wizard is done.\
        ",
            );
        let mut icon = Frame::default();
        icon.set_image(Some(PngImage::from_data(ICON_DATA).unwrap()));
        layout.fixed(&icon, 144);
        layout.end();
        main_layout.fixed(&layout, 182);

        const MAX_INSTALLS_COUNT: usize = 8;

        let mut layout = Flex::default().row();
        let tools_count_label = Frame::default().with_label("number");
        layout.fixed(&tools_count_label, 140);
        let mut tools_number_spinner = Spinner::default();
        layout.fixed(&tools_number_spinner, 48);
        tools_number_spinner.set_step(1.0);
        tools_number_spinner.set_minimum(0 as f64);
        tools_number_spinner.set_maximum(MAX_INSTALLS_COUNT as f64);
        layout.end();
        main_layout.fixed(&layout, ITEM_HEIGHT);

        let mut user_inputs = Vec::new();
        for _ in 0..MAX_INSTALLS_COUNT {
            let mut row_layout = Flex::default().row();
            let path_label = Frame::default().with_label("path to tools");
            row_layout.fixed(&path_label, 72);
            let tools_path = FileInput::default();
            let mut browse_btn = Button::default().with_label("browse");
            row_layout.end();
            row_layout.fixed(&browse_btn, 80);
            main_layout.fixed(&row_layout, ITEM_HEIGHT);

            row_layout.hide();

            // callbacks

            browse_btn.set_callback({
                let mut tools_path = tools_path.clone();
                move |_| {
                    let mut dialog =
                        NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseDir);
                    dialog.show();
                    let input_path = dialog.filename();
                    let input_str = &input_path.to_string_lossy();
                    tools_path.set_value(input_str);
                }
            });

            user_inputs.push((row_layout, tools_path));
        }
        main_layout.end();

        // initialize

        for ((layout, input), path) in user_inputs.iter_mut().zip(self.init_hfs_dirs.iter()) {
            layout.show();
            input.set_value(&path.to_string_lossy());
        }
        tools_number_spinner.set_value(self.init_hfs_dirs.len() as f64);
        self.widgets = Some(Rc::new(RefCell::new(user_inputs)));

        // callbacks

        tools_number_spinner.set_callback({
            let widgets = self.widgets.as_ref().unwrap().clone();
            move |w| {
                let number_of_versions = w.value() as usize;
                for i in 0..number_of_versions {
                    widgets.borrow_mut()[i].0.show();
                }
                for i in number_of_versions..MAX_INSTALLS_COUNT {
                    widgets.borrow_mut()[i].0.hide();
                }
                main_layout.layout();
            }
        });
        self.count_widget = Some(tools_number_spinner);
    }

    fn validate(&self) -> Result<(), &str> {
        if let Some(paths) = self.get_tools_install_locations() {
            for path in paths {
                if !path.is_absolute() {
                    return Err("path must be absolute");
                }
            }
        };
        Ok(())
    }

    fn contents_size(&self) -> (i32, i32) {
        (750, 500)
    }
}
