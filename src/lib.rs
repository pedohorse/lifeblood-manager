mod installation_data;
pub use installation_data::{InstallationsData, InstalledVersion};

#[cfg(feature = "ui")]
mod widgets;
#[cfg(feature = "ui")]
mod installation_widget;
#[cfg(feature = "ui")]
pub mod theme;
#[cfg(feature = "ui")]
pub use installation_widget::InstallationWidget;
#[cfg(feature = "ui")]
pub use widgets::Widget;
