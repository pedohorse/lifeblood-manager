use crate::widgets::{Widget, WidgetCallbacks};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use fltk::{
    prelude::*,
    group::Flex,
};

pub struct LaunchWidget {
    base_path: PathBuf,
}

impl WidgetCallbacks for LaunchWidget {
    fn install_location_changed(&mut self, path: &PathBuf){
        self.base_path = path.to_owned();
    }
}

impl Widget for LaunchWidget {
    fn initialize() -> Arc<Mutex<Self>> {
        let mut tab_header = Flex::default_fill().with_label("Launch\t").row();
        let mut flex = Flex::default_fill().column();
        flex.set_margin(8);
        flex.set_spacing(16);

        let widget = LaunchWidget {};

        flex.end();
        tab_header.end();

        Arc::new(Mutex::new(widget))
    }
}