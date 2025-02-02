use std::path::{Path, PathBuf};

use crate::info_dialog::InfoDialog;
use crate::wizard::wizard_data::{BlenderVersion, HoudiniVersion, RedshiftVersion};

use super::houdini_utils::possible_default_user_pref_dirs;
///
/// This is responsible for running wizard activities in proper order
/// and gathering parts for final configuration
///
use super::wizard_data::WizardData;
use super::wizard_data_from_config::WizardDataFromConfig;
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
    DoScratchPath,
    ChooseDCCs,
    FindBlender,
    FindHoudini,
    FindRedshift,
    HoudiniTools,
    GPUDevices,
    Finalize,
}

impl Wizard {
    pub fn new(config_root: PathBuf) -> Self {
        Wizard {
            data: WizardData::new_from_config(&config_root).unwrap_or_else(|_| WizardData::new_with_reasonable_defaults()),
            config_root,
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
                            self.state = WizardState::DoScratchPath;
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
                WizardState::DoScratchPath => {
                    let mut activity = if let Some(ref path) = self.data.scratch_path {
                        activities::scratchpath::ScratchLocationPathActivity::from_path(path)
                    } else {
                        activities::scratchpath::ScratchLocationPathActivity::new()
                    };

                    match runner.process(&mut activity) {
                        ActivityResult::Next => {
                            self.data.scratch_path = activity.selected_path();
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
                        self.data.do_redshift,
                    );
                    match runner.process(&mut activity) {
                        ActivityResult::Next => {
                            (
                                self.data.do_blender,
                                self.data.do_houdini,
                                self.data.do_redshift,
                            ) = activity.selected_dccs();
                            if self.data.do_blender {
                                self.state = WizardState::FindBlender;
                            } else if self.data.do_houdini {
                                self.state = WizardState::FindHoudini;
                            } else if self.data.do_redshift {
                                self.state = WizardState::FindRedshift;
                            } else {
                                self.state = WizardState::GPUDevices;
                            }
                        }
                        ActivityResult::Prev => {
                            self.state = WizardState::DoScratchPath;
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
                            } else if self.data.do_redshift {
                                self.state = WizardState::FindRedshift;
                            } else {
                                self.state = WizardState::GPUDevices;
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
                            if self.data.do_redshift {
                                self.state = WizardState::FindRedshift;
                            } else {
                                self.state = WizardState::GPUDevices;
                            }
                        }
                        ActivityResult::Prev => {
                            self.state = WizardState::FindHoudini;
                        }
                        ActivityResult::Abort => {
                            return;
                        }
                    }
                }
                WizardState::FindRedshift => {
                    let mut activity = activities::findredshift::FindRedshiftActivity::new(
                        self.data
                            .redshift_versions
                            .iter()
                            .map(|redshift| (redshift.bin_path.to_owned(), redshift.version))
                            .collect(),
                    );
                    match runner.process(&mut activity) {
                        ActivityResult::Next => {
                            if let Some(sel_vers) = activity.selected_versions() {
                                self.data.redshift_versions.clear();
                                for sel_ver in sel_vers.into_iter() {
                                    self.data.redshift_versions.push(RedshiftVersion {
                                        bin_path: sel_ver.0,
                                        version: sel_ver.1,
                                    });
                                }
                            }
                            self.state = WizardState::GPUDevices;
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
                WizardState::GPUDevices => {
                    let mut activity =
                        activities::gpudevices::GpuDevicesActivity::new(&self.data.gpu_devs);
                    match runner.process(&mut activity) {
                        ActivityResult::Next => {
                            let devs = activity.get_gpu_devices();
                            self.data.gpu_devs = devs;
                            self.state = WizardState::Finalize;
                        }
                        ActivityResult::Prev => {
                            if self.data.do_redshift {
                                self.state = WizardState::FindRedshift;
                            } else if self.data.do_houdini {
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
                WizardState::Finalize => {
                    // show summary
                    let mut activity = activities::summary::SummaryActivity::new(
                        true,
                        true,
                        self.data.db_path.as_deref(),
                        self.data.scratch_path.as_deref(),
                        &self.data.blender_versions,
                        &self.data.houdini_versions,
                        &self
                            .data
                            .houdini_plugins_installation_paths
                            .iter()
                            .map(|x| x as &Path)
                            .collect::<Vec<_>>(),
                        &self.data.gpu_devs,
                    );
                    match runner.process(&mut activity) {
                        ActivityResult::Next => {
                            break;
                        }
                        ActivityResult::Prev => {
                            self.state = WizardState::GPUDevices;
                        }
                        ActivityResult::Abort => {
                            return;
                        }
                    }
                }
            }
        }

        println!("executing wizardry...");
        if let Err(e) = self.data.execute_all_wizardry(&self.config_root) {
            eprintln!("error executing wizardry: {:?}", e);
            InfoDialog::show_in_center(
                "failed to execute wizardry :(",
                &format!("error occuerd: {:?}", e),
            );
            return;
        }
        InfoDialog::show_in_center(
            "The thread bundled by the laws of causality have now been bound.",
            "The Wizard has Succeeded",
        );
    }
}
