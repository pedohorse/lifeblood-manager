use std::sync::Mutex;
use std::sync::Arc;
use std::path::PathBuf;
use crate::installation_data::InstallationsData;
use crate::tray_manager::TrayManager;
use fltk::group::Flex;

pub trait WidgetCallbacks {
    fn install_location_changed(&mut self, path: &PathBuf, install_data: Option<&Arc<Mutex<InstallationsData>>>);
    fn on_tab_selected(&mut self);
    fn generate_tray_items(&mut self, tray_manager: &mut TrayManager);
}

pub trait Widget : WidgetCallbacks {
    fn initialize() -> (Arc<Mutex<Self>>, Flex);  // TODO: why the hell do i need arc mutex here??
}
