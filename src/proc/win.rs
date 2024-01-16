use std::{
    io,
    os::windows::process::CommandExt,
    path::Path,
    process::{Command, Child, Stdio},
};
use winconsole::console::generate_ctrl_event;

// const DETACHED_PROCESS: u32 = 0x00000008;
const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;  // This is needed for pid to be equal to group id
// const CREATE_NO_WINDOW: u32 = 0x08000000;


pub fn create_process(program: &str, args: &Vec<String>, cwd: &Path) -> io::Result<Child> {
    // rust likes working with "verbatim" paths,
    // but window's shell and some parts of python do not like such paths
    // so it's safer to just strip that shit
    let cwd = &{
        let tmp = cwd.to_str().unwrap();
        if tmp.starts_with("\\\\?\\") {
            Path::new(&tmp[4..])
        } else {
            cwd
        }
    };

    println!("starting {:?}", program);
    Command::new(cwd.join(program))
        .creation_flags(CREATE_NEW_PROCESS_GROUP)  // CREATE_NEW_PROCESS_GROUP is a must, other flats just break things for now
        .args(args)
        .stdin(Stdio::null())  // windows cmd loves to ask things like "Terminate batch job? Y/N". shutting stdin saves from that
        .current_dir(cwd)
        .spawn()
}

pub fn terminate_child(child: &Child) -> io::Result<()> {
    // generate_ctrl_event(true <- means send Ctrl+Break. with "false" it's supposed to be ctrl+c, but it just doesn't work at all
    match generate_ctrl_event(true, child.id()) {
        Ok(_) => return Ok(()),
        Err(e) => {
            eprintln!("error terminating process: {:?}", e);
            return Err(io::Error::new(io::ErrorKind::Other, "terminate failed"));
        }
    }
}