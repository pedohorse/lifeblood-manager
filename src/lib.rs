mod installation_data;
mod launch_data;
mod proc;
pub use installation_data::{InstallationsData, InstalledVersion};

#[cfg(feature = "ui")]
mod widgets;
#[cfg(feature = "ui")]
mod installation_widget;
#[cfg(feature = "ui")]
mod launch_widget;
#[cfg(feature = "ui")]
pub mod theme;
#[cfg(feature = "ui")]
pub use installation_widget::InstallationWidget;
#[cfg(feature = "ui")]
pub use launch_widget::LaunchWidget;
#[cfg(feature = "ui")]
pub use widgets::{Widget, WidgetCallbacks};
#[cfg(feature = "ui")]
mod info_dialog;
