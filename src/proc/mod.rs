#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::create_process;

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::{create_process, terminate_child};
