use crate::installation_data::InstallationsData;
use crate::main_widget_config::MainWidgetConfig;
use crate::tray_manager::TrayManager;
use fltk::group::Flex;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

pub trait WidgetCallbacks {
    fn install_location_changed(
        &mut self,
        path: &Path,
        install_data: Option<&Arc<Mutex<InstallationsData>>>,
    );
    fn on_tab_selected(&mut self);
    fn generate_tray_items(&mut self, tray_manager: &mut TrayManager);
    fn post_initialize(&mut self);
}

pub trait Widget: WidgetCallbacks {
    fn initialize(config: Rc<RefCell<MainWidgetConfig>>) -> (Arc<Mutex<Self>>, Flex); // TODO: why the hell do i need arc mutex here??
}
