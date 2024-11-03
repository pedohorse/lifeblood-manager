use home::home_dir;
use std::path::{Path, PathBuf};

use crate::info_dialog::InfoDialog;
use crate::wizard::wizard_data::{BlenderVersion, HoudiniVersion};

///
/// This is responsible for running wizard activities in proper order
/// and gathering parts for final configuration
///
use super::wizard_data::WizardData;
use super::wizard_data_to_config::*;
use super::{activities, wizard_activity::ActivityResult, wizard_activity_runner::ActivityRunner};

pub struct Wizard {
    config_root: PathBuf,
    data: WizardData,
    state: WizardState,
}

enum WizardState {
    Intro,
    DoDBPath,
    ChooseDCCs,
    FindBlender,
    FindHoudini,
    HoudiniTools,
    Finalize,
}

impl Wizard {
    pub fn new(config_root: PathBuf) -> Self {
        Wizard {
            config_root,
            data: WizardData::new(),
            state: WizardState::Intro,
        }
    }

    pub fn run(&mut self) {
        let mut runner = ActivityRunner::new();

        loop {
            match self.state {
                WizardState::Intro => {
                    let mut activity = activities::intro::IntroActivity::new();
                    match runner.process(&mut activity) {
                        ActivityResult::Next => {
                            self.state = WizardState::ChooseDCCs;
                        }
                        ActivityResult::Prev | ActivityResult::Abort => {
                            return;
                        }
                    }
                }
                WizardState::DoDBPath => {
                    // SKIPPED FOR NOW
                    let mut activity = if let Some(ref path) = self.data.db_path {
                        activities::dbpath::DBPathActivity::from_path(path)
                    } else {
                        activities::dbpath::DBPathActivity::new()
                    };

                    match runner.process(&mut activity) {
                        ActivityResult::Next => {
                            self.data.db_path = activity.selected_path();
                            self.state = WizardState::ChooseDCCs;
                        }
                        ActivityResult::Prev => {
                            self.state = WizardState::Intro;
                        }
                        ActivityResult::Abort => {
                            return;
                        }
                    }
                }
                WizardState::ChooseDCCs => {
                    let mut activity = activities::dcctypes::DCCTypesActivity::new(
                        self.data.do_blender,
                        self.data.do_houdini,
                    );
                    match runner.process(&mut activity) {
                        ActivityResult::Next => {
                            (self.data.do_blender, self.data.do_houdini) = activity.selected_dccs();
                            if self.data.do_blender {
                                self.state = WizardState::FindBlender;
                            } else if self.data.do_houdini {
                                self.state = WizardState::FindHoudini;
                            } else {
                                self.state = WizardState::Finalize;
                            }
                        }
                        ActivityResult::Prev => {
                            self.state = WizardState::Intro;
                        }
                        ActivityResult::Abort => {
                            return;
                        }
                    }
                }
                WizardState::FindBlender => {
                    let mut activity = activities::findblender::FindBlenderActivity::new(
                        self.data
                            .blender_versions
                            .iter()
                            .map(|v| (v.bin_path.to_owned(), v.version))
                            .collect(),
                    );
                    match runner.process(&mut activity) {
                        ActivityResult::Next => {
                            if let Some(sel_vers) = activity.selected_versions() {
                                self.data.blender_versions.clear();
                                for sel_ver in sel_vers.iter() {
                                    self.data.blender_versions.push(BlenderVersion {
                                        bin_path: sel_ver.0.to_owned(),
                                        version: sel_ver.1,
                                    })
                                }
                            }
                            if self.data.do_houdini {
                                self.state = WizardState::FindHoudini;
                            } else {
                                self.state = WizardState::Finalize;
                            }
                        }
                        ActivityResult::Prev => {
                            self.state = WizardState::ChooseDCCs;
                        }
                        ActivityResult::Abort => {
                            return;
                        }
                    }
                }
                WizardState::FindHoudini => {
                    let mut activity = activities::findhoudini::FindHoudiniActivity::new(
                        self.data
                            .houdini_versions
                            .iter()
                            .map(|v| (v.bin_path.to_owned(), v.version, v.python_version))
                            .collect(),
                    );
                    match runner.process(&mut activity) {
                        ActivityResult::Next => {
                            if let Some(sel_vers) = activity.selected_versions() {
                                self.data.houdini_versions.clear();
                                for sel_ver in sel_vers.iter() {
                                    self.data.houdini_versions.push(HoudiniVersion {
                                        bin_path: sel_ver.0.to_owned(),
                                        version: sel_ver.1,
                                        python_version: sel_ver.2,
                                    })
                                }
                            }
                            self.state = WizardState::HoudiniTools;
                        }
                        ActivityResult::Prev => {
                            if self.data.do_blender {
                                self.state = WizardState::FindBlender;
                            } else {
                                self.state = WizardState::ChooseDCCs;
                            }
                        }
                        ActivityResult::Abort => {
                            return;
                        }
                    }
                }
                WizardState::HoudiniTools => {
                    if !self.data.houdini_plugins_paths_first_initialized {
                        self.data.houdini_plugins_paths_first_initialized = true;
                        
                        // initial initialization
                        self.data.houdini_plugins_installation_paths = Vec::new();
                        if let Some(home_path) = home_dir() {
                            if let Ok(dir_iter) = home_path.read_dir() {
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
                                    if !dir_name.starts_with("houdini") || !dir_name.chars().skip(7).next().unwrap_or('x').is_numeric() {
                                        continue;
                                    }
                                    // assume entry is acceptible
                                    self.data.houdini_plugins_installation_paths.push(dir_entry.path());
                                }
                            }
                        }
                    }
                    let mut activity = activities::houdiniplugins::HoudiniToolsActivity::new(
                        self.data.houdini_plugins_installation_paths.clone(),
                    );
                    match runner.process(&mut activity) {
                        ActivityResult::Next => {
                            if let Some(tools_paths) = activity.get_tools_install_locations() {
                                self.data.houdini_plugins_installation_paths = tools_paths;
                            }
                            self.state = WizardState::Finalize;
                        }
                        ActivityResult::Prev => {
                            self.state = WizardState::FindHoudini;
                        }
                        ActivityResult::Abort => {
                            return;
                        }
                    }
                }
                WizardState::Finalize => {
                    // show summary
                    let mut activity = activities::summary::SummaryActivity::new(
                        self.data.db_path.as_deref(),
                        &self.data.blender_versions,
                        &self.data.houdini_versions,
                        &self.data.houdini_plugins_installation_paths.iter().map(|x| {x as &Path}).collect::<Vec<_>>(),
                    );
                    match runner.process(&mut activity) {
                        ActivityResult::Next => {
                            break;
                        }
                        ActivityResult::Prev => {
                            if self.data.do_houdini {
                                self.state = WizardState::HoudiniTools;
                            } else if self.data.do_blender {
                                self.state = WizardState::FindBlender;
                            } else {
                                self.state = WizardState::ChooseDCCs;
                            }
                        }
                        ActivityResult::Abort => {
                            return;
                        }
                    }
                }
            }
        }

        println!("saving config...");
        if let Err(e) = self.data.write_configs(&self.config_root) {
            eprintln!("error saving config: {:?}", e);
            InfoDialog::show_in_center(
                "failed to save config :(",
                &format!("error occuerd: {:?}", e),
            );
        }
    }
}
