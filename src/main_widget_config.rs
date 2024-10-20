use serde;
use std::collections::HashMap;
use std::fs;
use std::{
    collections::HashSet,
    io::Error,
    io::ErrorKind,
    path::{Path, PathBuf},
};
use toml;

#[derive(serde::Deserialize, serde::Serialize)]
enum ExtraFieldValue {
    String(String),
    Int(i64),
    StringList(Vec<String>),
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ConfigData {
    base_install_dir: PathBuf, // use this as main dir
    launch_ids_to_autostart: HashSet<String>,
    extra_fields: HashMap<String, ExtraFieldValue>,
    // additional launch data parameters may be stored here, or even the whole launch data?
}

pub struct MainWidgetConfig {
    config_path: Option<PathBuf>,
    config_data: ConfigData,
}

impl MainWidgetConfig {
    ///
    /// create new config, IGNORING anything already present in the given base dir
    pub fn new(base_dir: &Path) -> MainWidgetConfig {
        Self::new_full(Some(base_dir), base_dir)
    }

    pub fn new_ephemeral(base_dir: &Path) -> MainWidgetConfig {
        Self::new_full(None, base_dir)
    }

    fn new_full(config_dir: Option<&Path>, installation_dir: &Path) -> MainWidgetConfig {
        MainWidgetConfig {
            config_path: if let Some(dir) = config_dir {
                Some(dir.join("lifeblood-manager.config"))
            } else {
                None
            },
            config_data: ConfigData {
                base_install_dir: installation_dir.to_path_buf(),
                launch_ids_to_autostart: HashSet::new(),
                extra_fields: HashMap::new(),
            },
        }
    }

    ///
    /// load config from given base_install_dir location
    pub fn new_from_file(
        base_dir: &Path,
        empty_new_if_not_exists: bool,
        overwrite_if_broken: bool,
    ) -> Result<MainWidgetConfig, Error> {
        let mut config = Self::new(base_dir);
        match config.reload_from_file() {
            Err(e) => match e.kind() {
                ErrorKind::InvalidData if overwrite_if_broken => {}
                ErrorKind::NotFound if empty_new_if_not_exists => {}
                _ => {
                    return Err(e);
                }
            },
            Ok(_) => {}
        };

        Ok(config)
    }

    pub fn launch_ids_to_autostart(&self) -> &HashSet<String> {
        &self.config_data.launch_ids_to_autostart
    }

    pub fn has_autostart_launch_id(&self, launch_id: &str) -> bool {
        self.config_data.launch_ids_to_autostart.contains(launch_id)
    }

    pub fn add_autostart_launch_id(&mut self, launch_id: &str) {
        self.config_data
            .launch_ids_to_autostart
            .insert(launch_id.to_string());
    }

    pub fn remove_autostart_launch_id(&mut self, launch_id: &str) {
        self.config_data.launch_ids_to_autostart.remove(launch_id);
    }

    pub fn base_install_path(&self) -> &Path {
        &self.config_data.base_install_dir
    }

    pub fn set_base_install_path(&mut self, path: &Path) {
        self.config_data.base_install_dir = path.to_path_buf();
    }

    pub fn write_to_file(&self) -> Result<(), Error> {
        if let Some(ref config_file_path) = self.config_path {
            let config_string = match toml::to_string_pretty(&self.config_data) {
                Ok(s) => s,
                Err(e) => {
                    return Err(Error::new(ErrorKind::InvalidData, e));
                }
            };
            println!("[DEBUG] writing config to: {:?}", config_file_path);
            fs::write(config_file_path, config_string)?;
        } else {
            println!("[DEBUG] not saving ephemeral config");
        }
        Ok(())
    }

    pub fn reload_from_file(&mut self) -> Result<(), Error> {
        if let Some(ref config_file_path) = self.config_path {
            println!("[DEBUG] reading config from: {:?}", config_file_path);
            let config_text = fs::read_to_string(config_file_path)?;
            let config_data = match toml::from_str(&config_text) {
                Ok(conf) => conf,
                Err(e) => {
                    return Err(Error::new(ErrorKind::InvalidData, e.message()));
                }
            };
            self.config_data = config_data;
        } else {
            println!("[DEBUG] not reloading ephemeral config");
        }
        Ok(())
    }
}
