use semver::Version;
use std::fs::File;
use std::{
    fmt::Debug,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use toml;

///
/// lifeblood config has form of
/// <name>.toml file
/// <name>.d/ dir, where all toml files are treated as config parts
///
/// <name> is by default just "config"
///
/// for now config is hardcoded to be toml
pub struct ConfigData {
    name: String,
    main_config_file: PathBuf,
    additional_config_files: Vec<PathBuf>,
}

pub enum ConfigError {
    SyntaxError(String, Option<std::ops::Range<usize>>),
    SchemaError(String),
}

impl Debug for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SyntaxError(message, span_maybe) => {
                if let Some(span) = span_maybe {
                    f.write_fmt(format_args!(
                        "Syntax Error ({}): at [{}-{}]",
                        message, span.start, span.end
                    ))
                } else {
                    f.write_fmt(format_args!("Syntax Error ({})", message))
                }
            }
            Self::SchemaError(err) => f.write_fmt(format_args!("Schema Error: {}", err)),
        }
    }
}

#[derive(Debug)]
pub enum ConfigWritingError {
    ConfigError(ConfigError),
    IoError(io::Error),
}

#[derive(Debug)]
pub struct ConfigLoadError {
    pub access_error: Vec<PathBuf>,
    pub syntax_error: Vec<(PathBuf, (String, Option<std::ops::Range<usize>>))>,
    pub schema_error: Vec<PathBuf>,
}

impl ConfigLoadError {
    pub fn new() -> ConfigLoadError {
        ConfigLoadError {
            access_error: Vec::new(),
            syntax_error: Vec::new(),
            schema_error: Vec::new(),
        }
    }
}

impl ConfigData {
    /// lifeblood for now uses only base_name "config"
    pub fn load(base_path: &Path, base_name: &str) -> ConfigData {
        let mut main_path = base_path.join(Path::new(base_name));
        let mut d_path = main_path.clone();
        main_path.set_extension("toml");
        d_path.set_extension(".d");
        let mut d_paths = Vec::new();

        if d_path.exists() && d_path.is_dir() {
            for dir_entry_maybe in d_path.read_dir().unwrap() {
                if let Ok(dir_entry) = dir_entry_maybe {
                    d_paths.push(dir_entry.path());
                }
            }
        }
        d_paths.sort();

        ConfigData {
            name: base_name.to_string(),
            main_config_file: main_path,
            additional_config_files: d_paths,
        }
    }

    pub fn validate_config_text(config_text: &str) -> Result<(), ConfigError> {
        match config_text.parse::<toml::Table>() {
            Ok(table_root) => {
                // perform simple schema check
                if let Some(packages) = table_root.get("packages") {
                    if !packages.is_table() {
                        return Err(ConfigError::SchemaError(
                            "'package' key must be a mapping".to_string(),
                        ));
                    }
                    for (pkg_name, pkg_ver) in packages.as_table().unwrap().iter() {
                        // pkg_ver is supposed to be semantic version string
                        if let Some(ver_map) = pkg_ver.as_table() {
                            // now check version keys
                            for ver_str in ver_map.keys() {
                                if let Err(_) = Version::parse(ver_str) {
                                    return Err(ConfigError::SchemaError(format!(
                                        "package {} version {} is not a valid semantic version",
                                        pkg_name, ver_str
                                    )));
                                }
                            }
                        } else {
                            return Err(ConfigError::SchemaError(format!(
                                "package {} has version that is not a version mapping, but {:?}",
                                pkg_name, pkg_ver
                            )));
                        }
                    }
                }
                return Ok(());
            }
            Err(e) => {
                return Err(ConfigError::SyntaxError(e.message().to_string(), e.span()));
            } // TODO: check for schema error
        };
    }

    pub fn validate(&self) -> Result<(), ConfigLoadError> {
        let mut failed_to_reads = Vec::new();
        let mut syntax_errors = Vec::new();

        for file_path in [&self.main_config_file]
            .into_iter()
            .chain(self.additional_config_files.iter())
        {
            match File::open(file_path) {
                Ok(mut f) => {
                    let mut config_text = String::new();
                    if let Err(_e) = f.read_to_string(&mut config_text) {
                        failed_to_reads.push(file_path.as_path().to_path_buf());
                        continue;
                    }

                    if let Err(e) = config_text.parse::<toml::Table>() {
                        syntax_errors
                            .push((file_path.as_path().to_path_buf(), (e.message().to_string(), e.span())));
                        continue;
                    }

                    // TODO: schema checks
                }
                Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                    continue;
                }
                // generic error? assume failed to read for some reason
                Err(_) => {
                    failed_to_reads.push(file_path.as_path().to_path_buf());
                }
            }
        }

        if failed_to_reads.len() > 0 || syntax_errors.len() > 0 {
            return Err(ConfigLoadError {
                access_error: failed_to_reads,
                syntax_error: syntax_errors,
                schema_error: Vec::new(),
            });
        }

        Ok(())
    }

    pub fn main_config_text(&self) -> String {
        println!("opening {:?}", self.main_config_file);
        match File::open(&self.main_config_file) {
            Ok(mut f) => {
                let mut text = String::new();
                if let Err(_) = f.read_to_string(&mut text) {
                    // TODO: MAYBE some errors should be treated separately?
                    return String::new();
                }
                return text;
            }
            Err(_) => {
                return String::new();
            }
        }
    }

    pub fn set_main_config_text(&mut self, config_text: &str) -> Result<(), ConfigWritingError> {
        if let Err(e) = Self::validate_config_text(config_text) {
            return Err(ConfigWritingError::ConfigError(e));
        }

        let root_dir = self.main_config_file.parent().unwrap();
        if !root_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(root_dir) {
                return Err(ConfigWritingError::IoError(e));
            }
        }

        {
            let mut file = File::create(&self.main_config_file).unwrap();
            if let Err(e) = file.write_all(config_text.as_bytes()) {
                return Err(ConfigWritingError::IoError(e));
            }
        }

        Ok(())
    }

    pub fn set_additional_config_text(&mut self, additional_config_name: &str, config_text: &str) -> Result<(), ConfigWritingError> {
        if let Err(e) = Self::validate_config_text(config_text) {
            return Err(ConfigWritingError::ConfigError(e));
        }

        let root_dir = self.main_config_file.with_extension("d");
        if !root_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&root_dir) {
                return Err(ConfigWritingError::IoError(e));
            }
        }

        {
            let mut file = File::create(root_dir.join(additional_config_name).with_extension("toml")).unwrap();
            if let Err(e) = file.write_all(config_text.as_bytes()) {
                return Err(ConfigWritingError::IoError(e));
            }
        }

        Ok(())
    }

    pub fn main_config_path(&self) -> &Path {
        &self.main_config_file
    }
}
