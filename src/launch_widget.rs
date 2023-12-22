use crate::launch_data::LaunchedProcess;
use crate::proc::{create_process, terminate_child};
use crate::widgets::{Widget, WidgetCallbacks};
use crate::InstallationsData;
use fltk::button::Button;
use fltk::{app, frame::Frame, group::Flex, prelude::*};
use std::cell::RefCell;
use std::ffi::OsStr;
use std::io;
use std::path::{PathBuf, Component};
use std::process::{Child, Command};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

struct ControlButtonsData {
    process: Option<LaunchedProcess>,
    command_label: String,
    command: String,
    args: Vec<String>,
    current_installation: Option<Arc<Mutex<InstallationsData>>>,
    last_run_exit_code: Option<i32>,
    current_installation_changed_callback:
        Option<Box<dyn FnMut(&ControlButtonsData, Option<&InstallationsData>) -> ()>>, // arg will be new and prev installations data
}

impl ControlButtonsData {
    pub fn new(
        installations: Option<&Arc<Mutex<InstallationsData>>>,
        command_label: &str,
        command: &str,
        args: Vec<&str>,
    ) -> ControlButtonsData {
        ControlButtonsData {
            process: None,
            command_label: command_label.to_owned(),
            command: command.to_owned(),
            args: args.into_iter().map(|x| x.to_owned()).collect(),
            current_installation: if let Some(x) = installations {
                Some(x.clone())
            } else {
                None
            },
            last_run_exit_code: None,
            current_installation_changed_callback: None,
        }
    }

    pub fn install_location_changed(
        &mut self,
        install_data: Option<&Arc<Mutex<InstallationsData>>>,
    ) {
        let prev_installations_data = std::mem::replace(
            &mut self.current_installation,
            if let Some(installations) = install_data {
                Some(installations.clone())
            } else {
                None
            },
        );

        let mut callback_maybe =
            std::mem::replace(&mut self.current_installation_changed_callback, None);
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

        self.current_installation_changed_callback = callback_maybe;
    }
}

pub struct LaunchWidget {
    scheduler_launch_data: Rc<RefCell<ControlButtonsData>>,
    worker_pool_launch_data: Rc<RefCell<ControlButtonsData>>,
    viewer_launch_data: Rc<RefCell<ControlButtonsData>>,
}

impl WidgetCallbacks for LaunchWidget {
    fn install_location_changed(
        &mut self,
        _path: &PathBuf,
        install_data: Option<&Arc<Mutex<InstallationsData>>>,
    ) {
        self.scheduler_launch_data.borrow_mut().install_location_changed(install_data);
        self.worker_pool_launch_data.borrow_mut().install_location_changed(install_data);
        self.viewer_launch_data.borrow_mut().install_location_changed(install_data);
    }
}

impl Widget for LaunchWidget {
    fn initialize() -> (Arc<Mutex<Self>>, Flex) {
        let tab_header = Flex::default_fill().with_label("Launch\t").row();
        let mut flex = Flex::default_fill().column();
        flex.set_margin(8);
        flex.set_spacing(16);

        let scheduler_launch_data = Rc::new(RefCell::new(ControlButtonsData::new(
            None,
            "Scheduler",
            "./lifeblood",
            vec!["scheduler"],
        )));
        let wpool_launch_data = Rc::new(RefCell::new(ControlButtonsData::new(
            None,
            "Worker Pool",
            "./lifeblood",
            vec!["pool", "simple"],
        )));
        let viewer_launch_data = Rc::new(RefCell::new(ControlButtonsData::new(
            None,
            "Viewer",
            "./lifeblood_viewer",
            vec![],
        )));
        let mut widget = LaunchWidget {
            scheduler_launch_data: scheduler_launch_data.clone(),
            worker_pool_launch_data: wpool_launch_data.clone(),
            viewer_launch_data: viewer_launch_data.clone(),
        };
        widget.make_launch_buttons(&mut flex, scheduler_launch_data);
        widget.make_launch_buttons(&mut flex, wpool_launch_data);
        widget.make_launch_buttons(&mut flex, viewer_launch_data);

        flex.end();
        tab_header.end();

        (Arc::new(Mutex::new(widget)), tab_header)
    }
}

impl LaunchWidget {
    fn make_launch_buttons(
        &mut self,
        parent_group: &mut Flex,
        control_data: Rc<RefCell<ControlButtonsData>>,
    ) {
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
        Frame::default().with_label(&control_data.borrow().command_label);
        let mut info_label1 = Flex::default_fill().row();
        info_label1.fixed(&Frame::default().with_label("base:"), 48);
        let info_label_running_root = Frame::default().with_label("not running");
        info_label1.end();
        info_box.end();

        flex.end();

        // init state
        stop_button.deactivate();
        if let None = control_data.borrow_mut().current_installation {
            start_button.deactivate();
        }

        // ui callbacks
        let control_data_ref = control_data.clone();
        let mut start_button_cl = start_button.clone();
        let mut stop_button_cl = stop_button.clone();
        let mut status_label_cl = status_label.clone();
        let mut info_label_running_root_cl = info_label_running_root.clone();
        app::add_timeout3(1.0, move |handle| {
            let mut data = control_data_ref.borrow_mut();
            let proc = if let Some(ref mut x) = data.process {
                x
            } else {
                app::repeat_timeout3(2.0, handle);
                return;
            };

            match proc.try_wait() {
                Ok(Some(status)) => {
                    data.process = None;
                    let exit_code = status.code().unwrap_or(-1); // read code() help to see why we rewrap this option
                    data.last_run_exit_code = Some(exit_code);

                    match exit_code {
                        0 => status_label_cl.set_label("⚪ finished OK"),
                        -1 => status_label_cl.set_label("🔴 unhandled signal"),
                        1 => status_label_cl.set_label("🔴 generic error"),
                        2 => status_label_cl.set_label("🔴 argument error"),
                        x => status_label_cl.set_label(&format!("🔴 error code: {}", x)),
                    };
                    start_button_cl.activate();
                    stop_button_cl.deactivate();
                    info_label_running_root_cl.set_label("not running");
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
        let mut info_label_running_root_cl = info_label_running_root.clone();
        start_button.set_callback(move |_| {
            let mut data = control_data_ref.borrow_mut();
            if let Some(_) = data.process {
                eprintln!("start button: process already started!");
                return;
            }

            data.process = match data.current_installation {
                Some(ref installations) => {
                    match LaunchedProcess::new(installations, &data.command, &data.args) {
                        Ok(p) => {
                            info_label_running_root_cl.set_label(
                                &installations.lock().unwrap().base_path()
                                    .components()
                                    .rev()
                                    .take(2)
                                    .collect::<Vec<Component>>()
                                    .into_iter()
                                    .rev()
                                    .collect::<PathBuf>()
                                    .to_string_lossy()
                            );
                            Some(p)
                        },
                        Err(e) => {
                            eprintln!("failed to start process! {:?}", e);
                            let err = format!("🔴 failed to start! {}", e.kind());
                            status_label_cl.set_label(&err);
                            return;
                        }
                    }
                }
                None => {
                    return;
                }
            };
            start_button_cl.deactivate();
            stop_button_cl.activate();
            status_label_cl.set_label("🟢 running");
        });

        let control_data_ref = control_data.clone();
        let mut status_label_cl = status_label.clone();
        let mut stop_button_cl = stop_button.clone();
        stop_button.set_callback(move |_| {
            let data = control_data_ref.borrow_mut();
            if let Some(ref proc) = data.process {
                if let Err(e) = proc.send_terminate_signal() {
                    eprintln!("failed to call terminate on child process cuz of: {:?}", e);
                    return;
                }
                status_label_cl.set_label("🟠 terminating");
                stop_button_cl.deactivate();
            };
        });

        control_data
            .borrow_mut()
            .current_installation_changed_callback =
            Some(Box::new(move |data, old_install_data_maybe| {
                if let Some(_) = data.process {
                    // leave state change up to process finalizing
                    return;
                }

                if let Some(_) = old_install_data_maybe {
                    if let None = data.current_installation {
                        start_button.deactivate();
                        stop_button.activate();
                        status_label.set_label("⚪ invalid");
                    }
                } else {
                    if let Some(_) = data.current_installation {
                        start_button.activate();
                        stop_button.deactivate();
                        status_label.set_label("⚪ ready");
                    }
                }
            }));
    }
}
