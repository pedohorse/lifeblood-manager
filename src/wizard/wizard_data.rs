use std::path::{Path, PathBuf};

pub struct HoudiniVersion {
    pub bin_path: PathBuf,
    pub python_version: (u32, u32),
    pub version: (u32, u32, u32),
}

pub struct BlenderVersion {
    pub bin_path: PathBuf,
    pub version: (u32, u32, u32),
}

pub struct WizardData {
    pub db_path: Option<PathBuf>,
    pub do_houdini: bool,
    pub do_blender: bool,
    pub houdini_versions: Vec<HoudiniVersion>,
    pub blender_versions: Vec<BlenderVersion>,
    pub houdini_plugins_paths_first_initialized: bool,
    pub houdini_plugins_installation_paths: Vec<PathBuf>,
}

pub trait WizardDataSerialization {
    fn execute_all_wizardry(&self, config_root: &Path) -> Result<(), std::io::Error> {
        self.write_configs(config_root)?;
        self.install_tools()?;
        Ok(())
    }

    fn write_configs(&self, config_root: &Path) -> Result<(), std::io::Error>;
    fn install_tools(&self) -> Result<(), std::io::Error>;
}

impl WizardData {
    pub fn new() -> Self {
        WizardData {
            db_path: None,
            do_houdini: false,
            do_blender: false,
            houdini_versions: Vec::new(),
            blender_versions: Vec::new(),
            houdini_plugins_paths_first_initialized: false,
            houdini_plugins_installation_paths: Vec::new(),
        }
    }
}
