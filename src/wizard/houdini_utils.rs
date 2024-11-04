use home::home_dir;
use std::path::PathBuf;

/// return possible houdini user pref dirs
/// path construction logic is defined by https://www.sidefx.com/docs/houdini/basics/config_env.html#setting-environment-variables
/// HOUDINI_USER_PREF_DIR is not taken into account (for now)
pub fn possible_default_user_pref_dirs() -> Vec<PathBuf> {
    let mut ret = Vec::new();
    let home_base = if let Some(home) = home_dir() {
        home
    } else {
        return ret;
    };
    let base_dir = if cfg!(target_os = "windows") {
        home_base.join("Documents")
    } else if cfg!(target_os = "macos") {
        home_base.join("Library/Preferences/houdini")
    } else if cfg!(target_os = "linux") {
        home_base
    } else {
        unimplemented!("where am i ??");
    };
    if let Ok(dir_iter) = base_dir.read_dir() {
        for entry in dir_iter {
            let dir_entry = match entry {
                Ok(x) => x,
                Err(_) => {
                    continue;
                }
            };
            // filter out non-dirs
            match dir_entry.file_type() {
                Ok(x) => {
                    if !x.is_dir() {
                        continue;
                    }
                }
                Err(_) => {
                    continue;
                }
            }
            // filter by name
            let file_name = dir_entry.file_name();
            let dir_name = file_name.to_string_lossy();
            if cfg!(not(target_os = "macos")) {
                if !dir_name.starts_with("houdini")
                    || !dir_name.chars().skip(7).next().unwrap_or('x').is_numeric()
                {
                    continue;
                }
            } else {
                if !dir_name.chars().next().unwrap_or('x').is_numeric() {
                    continue;
                }
            }
            // assume entry is acceptible
            ret.push(dir_entry.path());
        }
    };
    ret
}
