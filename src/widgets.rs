use std::sync::Mutex;
use std::sync::Arc;

pub trait Widget {
    fn initialize() -> Arc<Mutex<Self>>;
}
