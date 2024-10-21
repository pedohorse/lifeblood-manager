use auto_launch::{AutoLaunch, Error};
use dunce;
use std::env::current_exe;

const APP_NAME: &'static str = "org.flow-of-causality.lifeblood-manager";

pub fn is_supported() -> bool {
    AutoLaunch::is_support() && current_exe().is_ok()
}

pub fn is_enabled() -> bool {
    get_dummy_auto_launch().is_enabled().unwrap() // it doesn't seem that implementation can generate errors for now
}

pub fn enable(args: &[impl AsRef<str>]) -> Result<(), Error> {
    let auto_starter = AutoLaunch::new(APP_NAME, &my_bin_path(), args);
    auto_starter.enable()
}

pub fn disable() -> Result<(), Error> {
    get_dummy_auto_launch().disable()
}

fn my_bin_path() -> String {
    dunce::simplified(&current_exe().unwrap())
        .to_string_lossy()
        .to_string()
}

fn get_dummy_auto_launch() -> AutoLaunch {
    AutoLaunch::new(APP_NAME, &my_bin_path(), &vec![""; 0])
}
