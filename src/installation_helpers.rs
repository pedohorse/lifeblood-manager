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
    let mut pypaths = Vec::new();
    if let Ok(x) = std::env::var("PYTHON_BIN") {
        pypaths.push(PathBuf::from(x));
    } else {
        pypaths.push(PathBuf::from("python"));
        pypaths.push(PathBuf::from("python3"));
    };

    for pypath in pypaths {
        match process::Command::new(&pypath).arg("--version").status() {
            Ok(status) => {
                if let Some(code) = status.code() {
                    #[cfg(windows)]
                    if code == 9009 {
                        // no idea - special windows exic tode meaning command not found?
                        continue;
                    }
                    // otherwise - pass
                } else {
                    continue;
                }
            }
            Err(_) => continue,
        }
        return Some(pypath)
    }

    None
}
