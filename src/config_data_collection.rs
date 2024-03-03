use std::path::{Path, PathBuf};
use crate::config_data::ConfigData;
use home::home_dir;

pub struct ConfigDataCollection {
    config_location: PathBuf
}

const PRODUCT_NAME: &str = "lifeblood";

/// represents all configuration data for lifeblood
/// passed location is supposed to be the root for configurations
/// for ex, default ~/lifeblood
impl ConfigDataCollection {
    pub fn default_config_location() -> PathBuf {
        match std::env::var("LIFEBLOOD_CONFIG_LOCATION") {
            Ok(value) => {
                PathBuf::from(value)
            }
            Err(_) => {
                let home = if let Some(d) = home_dir() { d } else {
                    return PathBuf::new();
                };
                
                // return default path
                match std::env::consts::OS {
                    "windows"|"linux" => home.join(PRODUCT_NAME),
                    "macos" => home.join("Library").join("Preferences").join(PRODUCT_NAME),
                    x => panic!("how did you manage to run this on os {} ??", x),
                }
            }
        }
    }
    
    pub fn new(config_location: &Path) -> ConfigDataCollection {
        ConfigDataCollection {
            config_location: config_location.to_owned()
        }
    }

    pub fn change_location(&mut self, new_location: &Path) {
        self.config_location = new_location.to_owned();
    }

    pub fn get_config_data(&self, config_name: &str) -> ConfigData {
        ConfigData::load(&self.config_location.join(config_name), "config")
    }
}