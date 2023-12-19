use crate::widgets::{Widget, WidgetCallbacks};
use crate::InstallationsData;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use fltk::button::Button;
use fltk::{
    prelude::*,
    group::Flex,
};

pub struct LaunchWidget {
}

impl WidgetCallbacks for LaunchWidget {
    fn install_location_changed(&mut self, path: &PathBuf, install_data: Option<&Arc<Mutex<InstallationsData>>>) {
        //self.base_path = path.to_owned();
    }
}

impl Widget for LaunchWidget {
    fn initialize() -> (Arc<Mutex<Self>>, Flex){
        let tab_header = Flex::default_fill().with_label("Launch\t").row();
        let mut flex = Flex::default_fill().column();
        flex.set_margin(8);
        flex.set_spacing(16);

        let widget = LaunchWidget {};
        Button::default_fill().with_label("testt");

        flex.end();
        tab_header.end();

        (Arc::new(Mutex::new(widget)), tab_header)
    }
}