use std::path::Path;

use crate::info_dialog::InfoDialog;

use super::houdini_utils::possible_default_user_pref_dirs;
use super::wizard_data::WizardData;
use super::wizard_data_to_config::*;
use super::{activities, wizard_activity::ActivityResult, wizard_activity_runner::ActivityRunner};

pub struct WizardForToolsOnly {
    data: WizardData,
    state: WizardState,
}

enum WizardState {
    HoudiniTools,
    Finalize,
}

impl WizardForToolsOnly {
    pub fn new() -> Self {
        WizardForToolsOnly {
            data: WizardData::new(),
            state: WizardState::HoudiniTools,
        }
    }

    pub fn run(&mut self) {
        let mut runner = ActivityRunner::new();

        loop {
            match self.state {
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
                            return;
                        }
                        ActivityResult::Abort => {
                            return;
                        }
                    }
                }
                WizardState::Finalize => {
                    // show summary
                    let mut activity = activities::summary::SummaryActivity::new(
                        false,
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
                            &[],
                    );
                    match runner.process(&mut activity) {
                        ActivityResult::Next => {
                            break;
                        }
                        ActivityResult::Prev => {
                            self.state = WizardState::HoudiniTools;
                        }
                        ActivityResult::Abort => {
                            return;
                        }
                    }
                }
            }
        }

        println!("installing houdini tools...");
        if let Err(e) = self.data.install_tools() {
            eprintln!("error installing tools: {:?}", e);
            InfoDialog::show_in_center(
                "failed to install tools :(",
                &format!("error occuerd: {:?}", e),
            );
        }
    }
}
