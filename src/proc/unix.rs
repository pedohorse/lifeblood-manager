use std::io;
use std::process::{Child, Command};

pub fn create_process(program: &str, args: &Vec<&str>) -> io::Result<Child> {
    Command::new(program).args(args).spawn()
}

pub fn terminate_child(child: &Child) -> io::Result<()> {
    // just use posix kill

    match Command::new("kill")
        .arg("-SIGTERM")
        .arg(child.id().to_string())
        .spawn()?
        .wait()
    {
        Ok(ec) if ec.code().unwrap_or(-1) == 0 => return Ok(()),
        Ok(_) => return Err(io::Error::new(io::ErrorKind::Other, "terminate failed")),
        Err(e) => return Err(e),
    }
}
