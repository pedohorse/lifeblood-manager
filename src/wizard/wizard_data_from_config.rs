use std::path::{Path, PathBuf};

use toml::Table;

use super::wizard_data::{BlenderVersion, HoudiniVersion, WizardData};
use super::wizard_data_serde_common::{EnvConfig, StringOrList, WorkerDevicesOnlyConfig};
use crate::config_data::ConfigLoadError;
use crate::config_data_collection::ConfigDataCollection;
use crate::wizard::wizard_data::RedshiftVersion;

pub trait WizardDataFromConfig {
    fn new_from_config(config_root: &Path) -> Result<WizardData, ConfigLoadError>;
}

impl WizardDataFromConfig for WizardData {
    fn new_from_config(config_root: &Path) -> Result<WizardData, ConfigLoadError> {
        let config_collection = ConfigDataCollection::new(config_root);
        let config = config_collection.get_config_data("standard_environment_resolver");

        // all syntax errors will be caught here
        config.validate()?;

        let config: EnvConfig = match toml::from_str(&config.main_config_text()) {
            Ok(c) => c,
            Err(e) => {
                let mut err = ConfigLoadError::new();
                err.schema_error.push((
                    config.main_config_path().to_path_buf(),
                    (e.message().to_owned(), e.span()),
                ));
                return Err(err);
            }
        };

        let mut wizard_data = WizardData::new();

        let scheduler_config_data = config_collection.get_config_data("scheduler");
        let worker_config_data = config_collection.get_config_data("worker");
        //
        // get autogen scratch location config
        let scratch_config_d_name = "00-autolbm-scratch-location";
        if let Some(text) = scheduler_config_data.additional_config_text(scratch_config_d_name) {
            let config: Table = match toml::from_str(&text) {
                Ok(c) => c,
                Err(e) => {
                    let mut err = ConfigLoadError::new();
                    // TODO: this can be syntax error!
                    err.schema_error.push((
                        scheduler_config_data
                            .additional_config_path(scratch_config_d_name)
                            .unwrap()
                            .to_owned(),
                        (e.message().to_owned(), e.span()),
                    ));
                    return Err(err);
                }
            };
            if let Some(config_sched) = config.get("scheduler") {
                if let Some(config_globals) = config_sched.get("globals") {
                    if let Some(toml::Value::String(s)) =
                        config_globals.get("global_scratch_location")
                    {
                        wizard_data.scratch_path = Some(PathBuf::from(s));
                    }
                }
            }
        }
        //
        // get autogen config devices
        let device_config_d_name = "10-autolbm-gpu-devices";
        if let Some(text) = worker_config_data.additional_config_text(&device_config_d_name) {
            let dev_config_data: WorkerDevicesOnlyConfig = match toml::from_str(&text) {
                Ok(c) => c,
                Err(e) => {
                    // since we checked for syntax error before (with validate()) - this can only be a schema error
                    let mut err = ConfigLoadError::new();
                    err.schema_error.push((
                        worker_config_data
                            .additional_config_path(&device_config_d_name)
                            .unwrap()
                            .to_owned(),
                        (e.message().to_owned(), e.span()),
                    ));
                    return Err(err);
                }
            };
            // now fill wizard data with it
            for (gpu_name, gpu_data) in dev_config_data.devices.gpu.into_iter() {
                wizard_data.gpu_devs.push((
                    gpu_name,
                    {
                        let text = gpu_data.resources.mem.unwrap_or_default();
                        u32::from_str_radix(
                            if text.ends_with("G") {
                                // we expect G suffix that we will trim
                                &text[0..text.len() - 1]
                            } else {
                                "0" // TODO: currently we don't know how to treat anything other than "G" suffix
                            },
                            10,
                        )
                        .unwrap_or(0)
                    },
                    gpu_data.resources.opencl_ver.unwrap_or(0.0),
                    gpu_data.resources.cuda_cc.unwrap_or(0.0),
                    gpu_data.tags.into_iter().collect(),
                ))
            }
        }

        //
        // packages
        for (package_name, ver_to_package) in config.packages.iter() {
            macro_rules! parse_or_skip {
                ($foo:expr) => {
                    match $foo.parse() {
                        Ok(s) => s,
                        Err(_) => continue,
                    }
                };
            }
            macro_rules! doo_foo {
                ($e:expr) => {
                    match $e {
                        StringOrList::String(ref s) => s,
                        StringOrList::List(ref l) => {
                            if l.len() != 1 {
                                continue;
                            }
                            &l[0]
                        }
                    }
                };
            }
            macro_rules! get_bin_path {
                ($package:ident) => {{
                    let env = match $package.env {
                        Some(ref e) => e,
                        None => continue,
                    };
                    let env_action = match env.get("PATH") {
                        Some(act) => act,
                        None => continue,
                    };

                    // we can only parse autogenerated entries, everything else is skipped
                    PathBuf::from({
                        if let Some(ref alist) = env_action.prepend {
                            doo_foo!(alist)
                        } else if let Some(ref alist) = env_action.append {
                            doo_foo!(alist)
                        } else {
                            // cannot understand this entry, so skip it
                            continue;
                        }
                    })
                }};
            }

            match package_name.as_str() {
                x if x.starts_with("houdini.py") || x.starts_with("houdini.") => {
                    let pname_offset = if package_name.starts_with("houdini.py") {
                        10
                    } else {
                        8
                    };
                    let pyver_parts: Vec<&str> = package_name[pname_offset..].split('_').collect();
                    if pyver_parts.len() != 2 {
                        continue;
                    }
    
                    let py_major: u32 = parse_or_skip!(pyver_parts[0]);
                    let py_minor: u32 = parse_or_skip!(pyver_parts[1]);
    
                    for (ver, package) in ver_to_package.iter() {
                        let bin_path = get_bin_path!(package);
    
                        let ver_parts: Vec<&str> = ver.split('.').collect();
                        if ver_parts.len() != 3 {
                            continue;
                        }
                        wizard_data.houdini_versions.push(HoudiniVersion {
                            bin_path,
                            python_version: (py_major, py_minor),
                            version: (
                                parse_or_skip!(ver_parts[0]),
                                parse_or_skip!(ver_parts[1]),
                                parse_or_skip!(ver_parts[2]),
                            ),
                        });
                    }
                }
                "blender" => {
                    for (ver, package) in ver_to_package.iter() {
                        let bin_path = get_bin_path!(package);
    
                        let ver_parts: Vec<&str> = ver.split('.').collect();
                        if ver_parts.len() != 3 {
                            continue;
                        }
                        wizard_data.blender_versions.push(BlenderVersion {
                            bin_path,
                            version: (
                                parse_or_skip!(ver_parts[0]),
                                parse_or_skip!(ver_parts[1]),
                                parse_or_skip!(ver_parts[2]),
                            ),
                        });
                    }
                }
                "redshift" => {
                    // NOTE: same shit as in blender
                    for (ver, package) in ver_to_package.iter() {
                        let bin_path = get_bin_path!(package);
    
                        let ver_parts: Vec<&str> = ver.split('.').collect();
                        if ver_parts.len() != 3 {
                            continue;
                        }
                        wizard_data.redshift_versions.push(RedshiftVersion {
                            bin_path,
                            version: (
                                parse_or_skip!(ver_parts[0]),
                                parse_or_skip!(ver_parts[1]),
                                parse_or_skip!(ver_parts[2]),
                            ),
                        });
                    }
                }
                s => {
                    eprintln!("unknown package type {s} is skipped and ignored");
                    // TODO: we should at least keep those packages
                }
            }
        }
        wizard_data.do_houdini = wizard_data.houdini_versions.len() > 0;
        wizard_data.do_blender = wizard_data.blender_versions.len() > 0;
        wizard_data.do_redshift = wizard_data.redshift_versions.len() > 0;

        Ok(wizard_data)
    }
}
