use crate::proc::{create_process, terminate_child};
use crate::InstallationsData;
use std::io;
use std::process::{Child, ExitStatus};
use std::sync::{Arc, Mutex};

pub struct LaunchedProcess {
    running_process: Child,
    original_installation: Arc<Mutex<InstallationsData>>,
}

impl LaunchedProcess {
    pub fn new(
        install_data: &Arc<Mutex<InstallationsData>>,
        program: &str,
        args: &Vec<String>,
    ) -> io::Result<LaunchedProcess> {
        let install_data = install_data.clone();
        let process = create_process(program, args, install_data.lock().unwrap().base_path())?;
        Ok(LaunchedProcess {
            running_process: process,
            original_installation: install_data,
        })
    }

    pub fn send_terminate_signal(&self) -> io::Result<()> {
        terminate_child(&self.running_process)
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.running_process.try_wait()
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.running_process.wait()
    }
}

