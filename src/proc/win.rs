use std::{
    io,
    os::windows::process::CommandExt,
    process::{Command, Child},
};
use winconsole::console::generate_ctrl_event;

const DETACHED_PROCESS: u32 = 0x00000008;
const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;  // This is needed for pid to be equal to group id
const CREATE_NO_WINDOW: u32 = 0x08000000;


pub fn create_process(program: &str, args: &Vec<&str>) -> io::Result<Child> {
    Command::new(program)
        .creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW)
        .args(args)
        .spawn()
}

pub fn terminate_child(child: &Child) -> io::Result<()> {
    match generate_ctrl_event(false, child.id()) {
        Ok(_) => return Ok(()),
        Err(e) => {
            eprintln!("error terminating process: {:?}", e);
            return Err(io::Error::new(io::ErrorKind::Other, "terminate failed"));
        }
    }
}