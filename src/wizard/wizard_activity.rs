#[derive(Clone, Copy)]
pub enum ActivityResult {
    Next,
    Prev,
    Abort,
}

pub trait WizardActivityTrait {
    ///
    /// start or REstart activity
    fn start_activity(&mut self);
    fn contents_size(&self) -> (i32, i32);

    ///
    /// validate current state of the activity.
    /// if error - error description is provided in a string
    fn validate(&self) -> Result<(), &str>;
}