use crate::launch_data::LaunchControlData;
use crate::widgets::{Widget, WidgetCallbacks};
use crate::InstallationsData;
use fltk::button::Button;
use fltk::{app, frame::Frame, group::Flex, prelude::*};
use std::cell::RefCell;
use std::path::{Component, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub struct LaunchWidget {
    scheduler_launch_data: Rc<RefCell<LaunchControlData>>,
    worker_pool_launch_data: Rc<RefCell<LaunchControlData>>,
    viewer_launch_data: Rc<RefCell<LaunchControlData>>,
}

impl WidgetCallbacks for LaunchWidget {
    fn install_location_changed(
        &mut self,
        _path: &PathBuf,
        install_data: Option<&Arc<Mutex<InstallationsData>>>,
    ) {
        self.scheduler_launch_data
            .borrow_mut()
            .install_location_changed(install_data);
        self.worker_pool_launch_data
            .borrow_mut()
            .install_location_changed(install_data);
        self.viewer_launch_data
            .borrow_mut()
            .install_location_changed(install_data);
    }
}

impl Widget for LaunchWidget {
    fn initialize() -> (Arc<Mutex<Self>>, Flex) {
        let tab_header = Flex::default_fill().with_label("Launch\t").row();
        let mut flex = Flex::default_fill().column();
        flex.set_margin(8);
        flex.set_spacing(16);

        let scheduler_launch_data = Rc::new(RefCell::new(LaunchControlData::new(
            None,
            "Scheduler",
            if cfg!(unix) { "./lifeblood" } else { "./lifeblood.cmd" },
            vec!["scheduler"],
        )));
        let wpool_launch_data = Rc::new(RefCell::new(LaunchControlData::new(
            None,
            "Worker Pool",
            if cfg!(unix) { "./lifeblood" } else { "./lifeblood.cmd" },
            vec!["pool", "simple"],
        )));
        let viewer_launch_data = Rc::new(RefCell::new(LaunchControlData::new(
            None,
            "Viewer",
            if cfg!(unix) { "./lifeblood_viewer" } else { "./lifeblood_viewer.cmd" },
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
        control_data: Rc<RefCell<LaunchControlData>>,
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
        Frame::default().with_label(&control_data.borrow().command_label());
        let mut info_label1 = Flex::default_fill().row();
        info_label1.fixed(&Frame::default().with_label("base:"), 48);
        let info_label_running_root = Frame::default().with_label("not running");
        info_label1.end();
        info_box.end();

        flex.end();

        // init state
        stop_button.deactivate();
        if !control_data.borrow_mut().is_current_installation_set() {
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
            if !data.is_process_running() {
                app::repeat_timeout3(2.0, handle);
                return;
            };

            match data.try_wait() {
                Ok(Some(status)) => {
                    let exit_code = status.code().unwrap_or(-1); // read code() help to see why we rewrap this option

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
            if let Some(_) = data.process() {
                eprintln!("start button: process already started!");
                return;
            }

            match data.start_process() {
                Ok(()) => {
                    info_label_running_root_cl.set_label(
                        &data
                            .current_installation()
                            .expect("unexpected: installation data disappeared!")
                            .base_path()
                            .components()
                            .rev()
                            .take(2)
                            .collect::<Vec<Component>>()
                            .into_iter()
                            .rev()
                            .collect::<PathBuf>()
                            .to_string_lossy(),
                    );
                }
                Err(e) => {
                    eprintln!("failed to start process! {:?}", e);
                    let err = format!("🔴 failed to start {}: {}", data.command(), e.kind());
                    status_label_cl.set_label(&err);
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
            if let Some(ref proc) = data.process() {
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
            .set_install_location_changed_callback(Some(Box::new(
                move |data, old_install_data_maybe| {
                    if data.is_process_running() {
                        // leave state change up to process finalizing
                        return;
                    }

                    if let Some(_) = old_install_data_maybe {
                        if !data.is_current_installation_set() {
                            start_button.deactivate();
                            stop_button.activate();
                            status_label.set_label("⚪ invalid");
                        }
                    } else {
                        if data.is_current_installation_set() {
                            start_button.activate();
                            stop_button.deactivate();
                            status_label.set_label("⚪ ready");
                        }
                    }
                },
            )));
    }
}
