use crate::proc::{create_process, terminate_child};
use crate::widgets::{Widget, WidgetCallbacks};
use crate::InstallationsData;
use fltk::button::Button;
use fltk::{app, frame::Frame, group::Flex, prelude::*};
use std::cell::RefCell;
use std::io;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

struct LaunchedProcess {
    running_process: Child,
    original_installation: Arc<Mutex<InstallationsData>>,
}

impl LaunchedProcess {
    pub fn new(
        install_data: &Arc<Mutex<InstallationsData>>,
        program: &str,
        args: &Vec<&str>,
    ) -> io::Result<LaunchedProcess> {
        let process = create_process(program, args)?;
        Ok(LaunchedProcess {
            running_process: process,
            original_installation: install_data.clone(),
        })
    }
}

struct ControlButtonsData {
    process: Option<LaunchedProcess>,
    command_name: String,
    current_installation: Arc<Mutex<InstallationsData>>,
    last_run_exit_code: Option<i32>,
}

impl ControlButtonsData {
    pub fn new(
        installations: Option<&Arc<Mutex<InstallationsData>>>,
        command_name: &str,
    ) -> ControlButtonsData {
        ControlButtonsData {
            process: None,
            command_name: command_name.to_owned(),
            current_installation: installations.clone(),
            last_run_exit_code: None,
        }
    }
}

pub struct LaunchWidget {
    scheduler_launch_data: ControlButtonsData
}

impl WidgetCallbacks for LaunchWidget {
    fn install_location_changed(
        &mut self,
        path: &PathBuf,
        install_data: Option<&Arc<Mutex<InstallationsData>>>,
    ) {
        //self.base_path = path.to_owned();
    }
}

impl Widget for LaunchWidget {
    fn initialize() -> (Arc<Mutex<Self>>, Flex) {
        let tab_header = Flex::default_fill().with_label("Launch\t").row();
        let mut flex = Flex::default_fill().column();
        flex.set_margin(8);
        flex.set_spacing(16);

        let scheduler_launch_data = ControlButtonsData::new(installations, command_name)
        let mut widget = LaunchWidget {};
        widget.make_launch_buttons(&mut flex);

        flex.end();
        tab_header.end();

        (Arc::new(Mutex::new(widget)), tab_header)
    }
}

impl LaunchWidget {
    fn make_launch_buttons(&mut self, parent_group: &mut Flex, control_data: ControlButtonsData) {
        let control_data = Rc::new(RefCell::new(control_data));

        let flex = Flex::default_fill().row();
        parent_group.fixed(&flex, 64);

        let button_box = Flex::default_fill().column();
        let mut status_label = Frame::default_fill().with_label("off");
        let button_group = Flex::default_fill().row();
        let mut start_button = Button::default_fill().with_label("start");
        let mut stop_button = Button::default_fill().with_label("stop");
        button_group.end();
        button_box.end();

        let info_box = Flex::default_fill().column();
        info_box.end();

        flex.end();

        // ui callbacks
        let control_data_ref = control_data.clone();
        let mut start_button_cl = start_button.clone();
        let mut stop_button_cl = stop_button.clone();
        let mut status_label_cl = status_label.clone();
        app::add_timeout3(1.0, move |handle| {
            let mut data = control_data_ref.borrow_mut();
            let proc = if let Some(ref mut x) = data.process {
                x
            } else {
                app::repeat_timeout3(2.0, handle);
                return;
            };

            match proc.running_process.try_wait() {
                Ok(Some(status)) => {
                    data.process = None;
                    let exit_code = status.code().unwrap_or(-1); // read code() help to see why we rewrap this option
                    data.last_run_exit_code = Some(exit_code);

                    match exit_code {
                        0 => status_label_cl.set_label("finished"),
                        -1 => status_label_cl.set_label("general error"),
                        1 => status_label_cl.set_label("generic error"),
                        2 => status_label_cl.set_label("argument error"),
                        x => status_label_cl.set_label(&format!("error code: {}", x)),
                    };
                    start_button_cl.activate();
                    stop_button_cl.deactivate();
                }
                Err(e) => {
                    eprintln!("failed to check process status: {:?}, ignoring", e);
                }
                Ok(None) => {} // we just wait
            };

            app::repeat_timeout3(1.0, handle);
        });

        let control_data_ref = control_data.clone();
        let mut start_button_cl = start_button.clone();
        let mut stop_button_cl = stop_button.clone();
        let mut status_label_cl = status_label.clone();
        start_button.set_callback(move |button| {
            let mut data = control_data_ref.borrow_mut();
            if let Some(_) = data.process {
                eprintln!("start button: process already started!");
                return;
            }
            data.process =
                match LaunchedProcess::new(&data.current_installation, "foo", &vec!["hmm..."]) {
                    Ok(p) => Some(p),
                    Err(e) => {
                        eprintln!("failed to start process! {:?}", e);
                        return;
                    }
                };
            start_button_cl.deactivate();
            stop_button_cl.activate();
            status_label_cl.set_label("running");
        });

        let control_data_ref = control_data.clone();
        let mut status_label_cl = status_label.clone();
        let mut stop_button_cl = stop_button.clone();
        stop_button.set_callback(move |_| {
            let data = control_data_ref.borrow_mut();
            if let Some(ref proc) = data.process {
                if let Err(e) = terminate_child(&proc.running_process) {
                    eprintln!("failed to call terminate on child process cuz of: {:?}", e);
                    return;
                }
                status_label_cl.set_label("terminating");
                stop_button_cl.deactivate();
            };
        });
    }
}
