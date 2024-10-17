use crate::running_process_data::LaunchedProcess;
use crate::InstallationsData;
use std::{
    io,
    process::ExitStatus,
    sync::{Arc, Mutex, MutexGuard},
};

pub enum LaunchControlDataOptionValueStorage {
    RawString(String),
    Int(i64),
    Enum((Vec<(String, String)>, usize)), // (key, label), selected option
    Nothing,
}

pub enum LaunchControlDataOptionValue {
    String(String),
    Int(i64),
    Enum(usize),
}

impl LaunchControlDataOptionValueStorage {
    pub fn new_enum(kvp: Vec<(&str, &str)>) -> LaunchControlDataOptionValueStorage {
        LaunchControlDataOptionValueStorage::Enum((
            kvp.into_iter().map(|(key, val)| {
                (key.to_owned(), val.to_owned())
            }).collect(),
            0
        ))
    }

    pub fn new_string(value: &str) -> LaunchControlDataOptionValueStorage {
        LaunchControlDataOptionValueStorage::RawString(value.to_owned())
    }

    pub fn new_int(value: i64) -> LaunchControlDataOptionValueStorage {
        LaunchControlDataOptionValueStorage::Int(value)
    }
}

pub struct LaunchControlDataOption {
    name: String,
    _storage: LaunchControlDataOptionValueStorage,
    _flag: Option<String>,
    _args_cache: Option<Vec<String>>,
}

impl LaunchControlDataOption {
    pub fn new(name: &str, option_value: LaunchControlDataOptionValueStorage, flag: Option<&str>) -> LaunchControlDataOption {
        LaunchControlDataOption {
            name: name.to_owned(),
            _storage: option_value,
            _flag: if let Some(x) = flag { Some(x.to_owned()) } else { None },
            _args_cache: None,
        }
    }

    pub fn label(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &LaunchControlDataOptionValueStorage {
        &self._storage
    }

    pub fn set_value(&mut self, new_value: LaunchControlDataOptionValue) {
        use LaunchControlDataOptionValueStorage::*;
        let mut tmp_val = LaunchControlDataOptionValueStorage::Nothing;
        std::mem::swap(&mut tmp_val, &mut self._storage);
        match (tmp_val, new_value) {
            (RawString(_), LaunchControlDataOptionValue::String(s)) => {
                self._storage = RawString(s);
            }
            (Int(_), LaunchControlDataOptionValue::Int(i)) => {
                self._storage = Int(i);
            }
            (Enum((menu, _)), LaunchControlDataOptionValue::Enum(i)) => {
                self._storage = Enum((menu, i));
            },
            _ => panic!("value type cannot be changed!")
        }
        self._args_cache = None;
    }

    pub fn get_args(&mut self) -> &[String] {
        if let None = self._args_cache {
            let mut args = Vec::with_capacity(2);
            if let Some(ref flag) = self._flag {
                args.push(flag.to_owned());
            }

            {
                use LaunchControlDataOptionValueStorage::*;
                match self._storage {
                    RawString(ref s) => {
                        args.push(s.to_owned());
                    }
                    Int(i) => {
                        args.push(format!("{}", i));
                    }
                    Enum((ref token_list, option_i)) => {
                        let (flag, _) = &token_list[option_i];
                        args.push(flag.to_owned());
                    }
                    Nothing => ()
                }
            }

            self._args_cache = Some(args);
        }

        if let Some(ref cache) = self._args_cache {
            return cache.as_slice();
        } else {
            unreachable!();
        }
    }
}

pub struct LaunchControlData {
    _id: String,
    _process: Option<LaunchedProcess>,
    _command_label: String,
    _description: String,
    _command: String,
    _args: Vec<String>,
    _args_options: Vec<LaunchControlDataOption>,
    _current_installation: Option<Arc<Mutex<InstallationsData>>>,
    _last_run_exit_code: Option<i32>,
    _current_installation_changed_callback:
        Option<Box<dyn FnMut(&LaunchControlData, Option<&InstallationsData>) -> ()>>, // arg will be new and prev installations data
}

impl LaunchControlData {
    ///
    /// launch_id MUST be unique
    ///
    pub fn new(
        launch_id: &str,
        installations: Option<&Arc<Mutex<InstallationsData>>>,
        command_label: &str,
        description: &str,
        command: &str,
        args: Vec<&str>,
        args_options: Option<Vec<LaunchControlDataOption>>,
    ) -> LaunchControlData {
        LaunchControlData {
            _id: launch_id.to_owned(),
            _process: None,
            _command_label: command_label.to_owned(),
            _description: description.to_owned(),
            _command: command.to_owned(),
            _args: args.into_iter().map(|x| x.to_owned()).collect(),
            _args_options: if let Some(v) = args_options {
                v
            } else {
                Vec::new()
            },
            _current_installation: if let Some(x) = installations {
                Some(x.clone())
            } else {
                None
            },
            _last_run_exit_code: None,
            _current_installation_changed_callback: None,
        }
    }

    pub fn args_options(&self) -> &Vec<LaunchControlDataOption> {
        &self._args_options
    }

    pub fn args_options_mut(&mut self) -> &mut Vec<LaunchControlDataOption> {
        &mut self._args_options
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

    pub fn set_install_location_changed_callback(
        &mut self,
        callback: Option<Box<dyn FnMut(&LaunchControlData, Option<&InstallationsData>) -> ()>>,
    ) {
        self._current_installation_changed_callback = callback;
    }

    pub fn start_process(&mut self) -> io::Result<()> {
        let mut args_full = self._args.clone();
        for opts in self._args_options.iter_mut() {
            args_full.extend_from_slice(opts.get_args());
        }
        
        println!("[DEBUG] about to start: {} with args {:?}", self._command, args_full);
        self._process = match self._current_installation {
            Some(ref installations) => {
                match LaunchedProcess::new(
                    installations.lock().unwrap().base_path(),
                    &self._command,
                    &args_full,
                ) {
                    Ok(p) => Some(p),
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "installation data is not set",
                ));
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

    pub fn process_pid(&self) -> Option<u32> {
        if let Some(ref proc) = self._process {
            Some(proc.pid())
        } else {
            None
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
            None => Err(io::Error::new(io::ErrorKind::Other, "not started")),
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
            None => Err(io::Error::new(io::ErrorKind::Other, "not started")),
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
            Some(ref mutexed_installations) => Some(mutexed_installations.lock().unwrap()),
            None => None,
        }
    }

    pub fn process(&self) -> Option<&LaunchedProcess> {
        self._process.as_ref()
    }

    pub fn command(&self) -> &str {
        &self._command
    }

    pub fn launch_id(&self) -> &str {
        &self._id
    }

    pub fn command_label(&self) -> &str {
        &self._command_label
    }

    pub fn description(&self) -> &str {
        &self._description
    }

    pub fn last_run_exit_code(&self) -> Option<i32> {
        self._last_run_exit_code
    }
}
