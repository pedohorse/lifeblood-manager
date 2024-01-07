use crate::proc::{create_process, terminate_child};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, ExitStatus};
use std::thread::sleep;
use std::time::Duration;

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

    pub fn pid(&self) -> u32 {
        self.running_process.id()
    }
}

impl Drop for LaunchedProcess {
    fn drop(&mut self) {
        println!("[INFO] managed process still running at exit, terminating...");
        if let Ok(None) = self.running_process.try_wait() {
            if let Err(e) = self.send_terminate_signal() {
                eprintln!("[ERROR] failed to send terminate to running process, trying to kill instead: {}", e);
                if let Err(e) = self.running_process.kill() {
                    eprintln!("[ERROR] failed to kill running process, process may still be running: {}", e);
                    return;
                }
            }
            
            'wait: {
                for _ in 0..30 {
                    sleep(Duration::from_millis(500));
                    match self.try_wait() {
                        Ok(Some(_)) => break 'wait,
                        Ok(None) => continue,
                        Err(e) => eprintln!("[ERROR] failed to wait for managed process, process may still be running: {}", e),
                    }
                }
                println!("[WARNING] managed process still running after terminate attempt, killing...");
                if let Err(e) = self.running_process.kill() {
                    eprintln!("[ERROR] failed to kill managed process, process may still be running: {}", e);
                }
            }
        }
        println!("[INFO] managed process stopped.");
    }
}