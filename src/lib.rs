mod installation_data;
mod widgets;
mod installation_widget;

pub mod theme;
pub use installation_data::{InstallationsData, InstalledVersion};
pub use installation_widget::InstallationWidget;
pub use widgets::Widget;