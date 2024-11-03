use std::path::{Path, PathBuf};

use crate::info_dialog::InfoDialog;
use crate::wizard::wizard_data::{BlenderVersion, HoudiniVersion};

use super::houdini_utils::possible_default_user_pref_dirs;
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
                        self.data.houdini_plugins_installation_paths =
                            possible_default_user_pref_dirs();
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
                        &self
                            .data
                            .houdini_plugins_installation_paths
                            .iter()
                            .map(|x| x as &Path)
                            .collect::<Vec<_>>(),
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
