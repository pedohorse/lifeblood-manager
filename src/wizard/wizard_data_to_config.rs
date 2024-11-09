use super::wizard_data::WizardData;
pub use super::wizard_data::WizardDataSerialization;
use super::wizard_data_serde_common::{EnvAction, EnvConfig, Package, StringOrList};
use crate::{config_data::ConfigWritingError, config_data_collection::ConfigDataCollection};
use downloader::{Download, Downloader};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufReader, Error},
    path::Path,
    time::Duration,
};
use tempfile::tempdir;
use zip::ZipArchive;

impl WizardDataSerialization for WizardData {
    fn write_configs(&self, config_root: &Path) -> Result<(), io::Error> {
        let config_collection = ConfigDataCollection::new(config_root);

        if let Some(_) = self.db_path {
            panic!("DATABASE CONFIGURATION IS NOT YET IMPLEMENTED")
        }

        if let Some(path) = &self.scratch_path {
            let mut config_data = config_collection.get_config_data("scheduler");
            let mut conf = toml::Table::new();
            let mut conf_sched = toml::Table::new();
            let mut conf_globals = toml::Table::new();
            conf_globals.insert(
                "global_scratch_location".to_string(),
                toml::Value::String(path.to_string_lossy().to_string()),
            );
            conf_sched.insert("globals".to_string(), toml::Value::Table(conf_globals));
            conf.insert("scheduler".to_string(), toml::Value::Table(conf_sched));

            let text = match toml::to_string_pretty(&conf) {
                Ok(x) => x,
                Err(_) => panic!("unexpected internal error!"),
            };
            match config_data.set_additional_config_text("00-autolbm-scratch-location", &text) {
                Err(ConfigWritingError::IoError(e)) => return Err(e),
                Err(e) => return Err(Error::new(io::ErrorKind::Other, format!("{:?}", e))),
                _ => (),
            }
        } else {
            // otherwise we should delete autocreated config files if any
            let mut config_data = config_collection.get_config_data("scheduler");
            config_data.remove_additional_config("00-autolbm-scratch-location")?;
        }

        let mut config = config_collection.get_config_data("standard_environment_resolver");

        let mut conf_blender_vers = HashMap::new();
        for ver in self.blender_versions.iter() {
            conf_blender_vers.insert(
                format!("{}.{}.{}", ver.version.0, ver.version.1, ver.version.2),
                Package {
                    label: Some("Blender".to_owned()),
                    env: Some(HashMap::from([(
                        "PATH".to_owned(),
                        EnvAction {
                            append: None,
                            prepend: Some(StringOrList::String(
                                ver.bin_path.to_string_lossy().to_string(),
                            )),
                            set: None,
                        },
                    )])),
                },
            );
        }

        let mut conf_packages = HashMap::new();
        for ver in self.houdini_versions.iter() {
            let hou_package_name = format!(
                "houdini.py{}_{}",
                ver.python_version.0, ver.python_version.1
            );
            if !conf_packages.contains_key(&hou_package_name) {
                conf_packages.insert(hou_package_name.to_owned(), HashMap::new());
            }
            let conf_packages_vers = conf_packages.get_mut(&hou_package_name).unwrap();
            conf_packages_vers.insert(
                format!("{}.{}.{}", ver.version.0, ver.version.1, ver.version.2),
                Package {
                    label: Some(format!(
                        "SideFX Houdini version {}.{}.{}",
                        ver.version.0, ver.version.1, ver.version.2
                    )),
                    env: Some(HashMap::from([(
                        "PATH".to_owned(),
                        EnvAction {
                            append: None,
                            prepend: Some(StringOrList::String(
                                ver.bin_path.to_string_lossy().to_string(),
                            )),
                            set: None,
                        },
                    )])),
                },
            );
        }
        if conf_blender_vers.len() > 0 {
            conf_packages.insert("blender".to_owned(), conf_blender_vers);
        }

        match toml::to_string_pretty(&EnvConfig {
            packages: conf_packages,
        }) {
            Ok(text) => match config.set_main_config_text(&text) {
                Err(ConfigWritingError::IoError(e)) => return Err(e),
                Err(e) => return Err(Error::new(io::ErrorKind::Other, format!("{:?}", e))),
                _ => (),
            },
            Err(_) => panic!("unexpected internal error!"),
        };
        Ok(())
    }

    fn install_tools(&self) -> Result<(), std::io::Error> {
        // download and unpack latest release of tools/plugins
        if self.houdini_plugins_installation_paths.len() > 0 {
            let temp_location = tempdir()?;
            let download_location = temp_location.path().join("arch");
            fs::create_dir(&download_location)?;
            let tools_location = temp_location.path().join("tools");
            fs::create_dir(&tools_location)?;

            //download
            let mut downloader = match Downloader::builder()
                .connect_timeout(Duration::from_secs(60))
                .timeout(Duration::from_secs(600))
                .retries(5)
                .download_folder(&download_location)
                .build()
            {
                Ok(x) => x,
                Err(e) => {
                    return Err(Error::new(
                        io::ErrorKind::Other,
                        format!("failed to create a downloader: {:?}", e),
                    ));
                }
            };
            println!("[INFO] downloading tools from github...");
            if let Err(e) = downloader.download(&[Download::new(
                "https://github.com/pedohorse/lifeblood/releases/latest/download/houdini.zip",
            )]) {
                return Err(Error::new(
                    io::ErrorKind::Other,
                    format!("failed to download houdini tools: {:?}", e),
                ));
            }
            let tools_archive_path = download_location.join("houdini.zip"); // TODO: get actual path from downloader, just in case

            // unzip
            println!("[INFO] extracting tools archive...");
            let reader = BufReader::new(match File::open(&tools_archive_path) {
                Ok(f) => f,
                Err(e) => {
                    return Err(Error::new(
                        e.kind(),
                        format!("failed to read downloaded archive: {}", e),
                    ));
                }
            });
            let mut arch = match ZipArchive::new(reader) {
                Ok(x) => x,
                Err(e) => {
                    return Err(Error::new(
                        io::ErrorKind::Other,
                        format!("error reading zip file: {}", e),
                    ));
                }
            };
            if let Err(e) = arch.extract(&tools_location) {
                return Err(Error::new(
                    io::ErrorKind::Other,
                    format!("failed to extract files from tools zip: {}", e),
                ));
            }

            // now ready to copy
            for plugin_base_path in self.houdini_plugins_installation_paths.iter() {
                println!("[INFO] copying houdini tools to: {:?}", plugin_base_path);
                let mut options = fs_extra::dir::CopyOptions::new();
                options.overwrite = true;
                options.content_only = true;
                if let Err(e) = fs_extra::dir::copy(&tools_location, plugin_base_path, &options) {
                    return Err(Error::new(
                        io::ErrorKind::Other,
                        format!("error copying houdini tools: {}", e),
                    ));
                }
            }
        }

        Ok(())
    }
}
