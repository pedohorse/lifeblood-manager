use std::path::PathBuf;
use std::process;

///
/// helper func
///
/// find python executable
///
/// usese PYTHON_BIN env variable
/// if not set - assumes standard `python` command
/// then tries to determine validity by simply running `python --version`
///
/// if a valid working python was found - it's path is returned
/// otherwise - None
///
pub fn get_python_command() -> Option<PathBuf> {
    // TODO: do checks, use env variable or smth
    //  propagate errors
    let pypath = if let Ok(x) = std::env::var("PYTHON_BIN") {
        PathBuf::from(x)
    } else {
        PathBuf::from("python")
    };

    match process::Command::new(&pypath).arg("--version").output() {
        Ok(output) => {
            if let Some(code) = output.status.code() {
                #[cfg(windows)]
                if code == 9009 {
                    // no idea - special windows exic tode meaning command not found?
                    return None;
                }
                // otherwise - pass
            } else {
                return None;
            }
        }
        Err(_) => return None,
    }

    Some(pypath)
}
