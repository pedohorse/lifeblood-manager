pub const BUILD_INFO: &'static str = if let Some(x) = option_env!("LM_BUILD_VERSION") { x } else {"UNKNOWN BUILD"};

mod installation_data;
mod running_process_data;
mod launch_data;
mod proc;
mod wizard;
pub mod config_data;
pub mod installation_helpers;
pub mod config_data_collection;
pub use installation_data::{InstallationsData, InstalledVersion};
pub use launch_data::LaunchControlData;
pub use running_process_data::LaunchedProcess;

#[cfg(feature = "ui")]
mod widgets;
#[cfg(feature = "ui")]
mod installation_widget;
#[cfg(feature = "ui")]
mod launch_widget;
#[cfg(feature = "ui")]
mod envres_config_widget;
#[cfg(feature = "ui")]
pub mod theme;
#[cfg(feature = "ui")]
pub use installation_widget::InstallationWidget;
#[cfg(feature = "ui")]
pub use launch_widget::LaunchWidget;
#[cfg(feature = "ui")]
pub use envres_config_widget::StandardEnvResolverConfigWidget;
#[cfg(feature = "ui")]
pub use widgets::{Widget, WidgetCallbacks};
#[cfg(feature = "ui")]
mod info_dialog;
