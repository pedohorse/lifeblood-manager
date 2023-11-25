use std::sync::Mutex;
use std::sync::Arc;
use std::path::PathBuf;

pub trait WidgetCallbacks {
    fn install_location_changed(&mut self, path: &PathBuf);
}

pub trait Widget : WidgetCallbacks {
    fn initialize() -> Arc<Mutex<Self>>;
}
