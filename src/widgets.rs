use std::sync::Mutex;
use std::sync::Arc;
use std::path::PathBuf;
use crate::installation_data::InstallationsData;
use fltk::group::Flex;

pub trait WidgetCallbacks {
    fn install_location_changed(&mut self, path: &PathBuf, install_data: Option<&Arc<Mutex<InstallationsData>>>);
}

pub trait Widget : WidgetCallbacks {
    fn initialize() -> (Arc<Mutex<Self>>, Flex);  // TODO: why the hell do i need arc mutex here??
}
