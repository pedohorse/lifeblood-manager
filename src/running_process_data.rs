use crate::proc::{create_process, terminate_child};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, ExitStatus};

pub struct LaunchedProcess {
    running_process: Child,
    original_installation_path: PathBuf,
}

impl LaunchedProcess {
    pub fn new(
        install_data_path: &Path,
        program: &str,
        args: &Vec<String>,
    ) -> io::Result<LaunchedProcess> {
        let process = create_process(program, args, install_data_path)?;
        Ok(LaunchedProcess {
            running_process: process,
            original_installation_path: PathBuf::from(install_data_path),
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

    pub fn base_path(&self) -> &Path {
        &self.original_installation_path
    }
}
