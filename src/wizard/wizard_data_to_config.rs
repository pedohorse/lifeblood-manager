use super::wizard_data::WizardData;
pub use super::wizard_data::WizardDataSerialization;
use crate::{config_data::ConfigWritingError, config_data_collection::ConfigDataCollection};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{self, Error}, path::Path,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum StringOrList {
    String(String),
    List(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
struct EnvConfig {
    packages: HashMap<String, HashMap<String, Package>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Package {
    label: Option<String>,
    env: Option<HashMap<String, EnvAction>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EnvAction {
    append: Option<StringOrList>,
    prepend: Option<StringOrList>,
    set: Option<String>,
}

impl WizardDataSerialization for WizardData {
    fn write_configs(&self, config_root: &Path) -> Result<(), io::Error> {
        let config_collection = ConfigDataCollection::new(config_root);

        if let Some(_) = self.db_path {
            panic!("DATABASE CONFIGURATION IS NOT YET IMPLEMENTED")
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
            let hou_package_name =
                format!("houdini.{}_{}", ver.python_version.0, ver.python_version.1);
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
}

#[test]
fn basic() {
    let conf_text = "\
    [packages.\"houdini.py3\".\"19.0.720\"]\n\
    label = \"SideFX Houdini, with python version 3\"\n\
    env.PATH.prepend = \"/home/xapkohheh/sw/result/houdinii-19.0.720/bin\"\n\
    \n\
    [packages.houdini.\"19.5.569\"]\n\
    label = \"SideFX Houdini, with python version 3\"\n\
    env.PATH.prepend = [\"/home/xapkohheh/sw/result/houdini-19.5.569/bin\"]\n\
    ";

    let config: EnvConfig = toml::from_str(conf_text).unwrap();

    println!("{:?}", config);
}
