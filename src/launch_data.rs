use crate::running_process_data::LaunchedProcess;
use crate::InstallationsData;
use std::{sync::{Arc, Mutex, MutexGuard}, io, process::ExitStatus};

pub struct LaunchControlData {
    _process: Option<LaunchedProcess>,
    _command_label: String,
    _command: String,
    _args: Vec<String>,
    _current_installation: Option<Arc<Mutex<InstallationsData>>>,
    _last_run_exit_code: Option<i32>,
    _current_installation_changed_callback:
        Option<Box<dyn FnMut(&LaunchControlData, Option<&InstallationsData>) -> ()>>, // arg will be new and prev installations data
}

impl LaunchControlData {
    pub fn new(
        installations: Option<&Arc<Mutex<InstallationsData>>>,
        command_label: &str,
        command: &str,
        args: Vec<&str>,
    ) -> LaunchControlData {
        LaunchControlData {
            _process: None,
            _command_label: command_label.to_owned(),
            _command: command.to_owned(),
            _args: args.into_iter().map(|x| x.to_owned()).collect(),
            _current_installation: if let Some(x) = installations {
                Some(x.clone())
            } else {
                None
            },
            _last_run_exit_code: None,
            _current_installation_changed_callback: None,
        }
    }

    pub fn install_location_changed(
        &mut self,
        install_data: Option<&Arc<Mutex<InstallationsData>>>,
    ) {
        let prev_installations_data = std::mem::replace(
            &mut self._current_installation,
            if let Some(installations) = install_data {
                Some(installations.clone())
            } else {
                None
            },
        );

        let mut callback_maybe =
            std::mem::replace(&mut self._current_installation_changed_callback, None);
        // soooo, callback won't be able to see itself on control data..?
        //  a bit strange this way, but i guess we can go with it

        if let Some(ref mut callback) = callback_maybe {
            //let prev_data =
            if let Some(ref x) = prev_installations_data {
                callback(self, Some(&x.lock().expect("failed to lock")));
            } else {
                callback(self, None);
            };
        };

        self._current_installation_changed_callback = callback_maybe;
    }

    pub fn set_install_location_changed_callback(&mut self, callback: Option<Box<dyn FnMut(&LaunchControlData, Option<&InstallationsData>) -> ()>>) {
        self._current_installation_changed_callback = callback;
    }

    pub fn start_process(&mut self) -> io::Result<()> {
        self._process = match self._current_installation {
            Some(ref installations) => {
                match LaunchedProcess::new(installations.lock().unwrap().base_path(), &self._command, &self._args) {
                    Ok(p) => {
                        Some(p)
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            None => {
                return Err(io::Error::new(io::ErrorKind::Other, "installation data is not set"));
            }
        };
        Ok(())
    }

    pub fn is_process_running(&self) -> bool {
        if let Some(_) = self._process {
            true
        } else {
            false
        }
    }
    
    /// TODO: add docstrings explaining behaviour
    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        match self._process {
            Some(ref mut proc) => {
                let status_maybe = proc.try_wait();
                if let Ok(Some(exit_status)) = status_maybe {
                    self._process = None;
                    let exit_code = exit_status.code();
                    self._last_run_exit_code = exit_code;
                }
                status_maybe
            }
            None => Err(io::Error::new(io::ErrorKind::Other, "not started"))
        }
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        match self._process {
            Some(ref mut proc) => {
                let status = proc.wait();
                if let Ok(exit_status) = status {
                    self._process = None;
                    let exit_code = exit_status.code();
                    self._last_run_exit_code = exit_code;
                }
                status
            }
            None => Err(io::Error::new(io::ErrorKind::Other, "not started"))
        }
    }

    pub fn is_current_installation_set(&self) -> bool {
        if let Some(_) = self._current_installation {
            true
        } else {
            false
        }
    }

    pub fn current_installation(&self) -> Option<MutexGuard<InstallationsData>> {
        match self._current_installation {
            Some(ref mutexed_installations) => {
                Some(mutexed_installations.lock().unwrap())
            }
            None => None
        }
    }

    pub fn process(&self) -> Option<&LaunchedProcess> {
        self._process.as_ref()
    }

    pub fn command(&self) -> &str {
        &self._command
    }

    pub fn command_label(&self) -> &str {
        &self._command_label
    }

    pub fn last_run_exit_code(&self) -> Option<i32> {
        self._last_run_exit_code
    }
}
