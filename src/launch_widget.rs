use crate::launch_data::{
    LaunchControlData, LaunchControlDataOption, LaunchControlDataOptionValueStorage,
};
use crate::theme::ITEM_HEIGHT;
use crate::widgets::{Widget, WidgetCallbacks};
use crate::InstallationsData;
use fltk::button::Button;
use fltk::enums::{Align, CallbackTrigger};
use fltk::input::{Input, IntInput};
use fltk::menu::Choice;
use fltk::{app, frame::Frame, group::Flex, prelude::*};
use std::cell::RefCell;
use std::path::{Component, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub struct LaunchWidget {
    launch_datas: Vec<Rc<RefCell<LaunchControlData>>>,
}

impl WidgetCallbacks for LaunchWidget {
    fn install_location_changed(
        &mut self,
        _path: &PathBuf,
        install_data: Option<&Arc<Mutex<InstallationsData>>>,
    ) {
        for launch_data in self.launch_datas.iter_mut() {
            launch_data
                .borrow_mut()
                .install_location_changed(install_data);
        }
    }

    fn on_tab_selected(&mut self) {}
}

impl Widget for LaunchWidget {
    fn initialize() -> (Arc<Mutex<Self>>, Flex) {
        let tab_header = Flex::default_fill().with_label("Launch\t").row();
        let mut flex = Flex::default_fill().column();
        flex.set_margin(8);
        flex.set_spacing(16);

        // different launch options
        let scheduler_launch_data = Rc::new(RefCell::new(LaunchControlData::new(
            None,
            "Scheduler",
            "This should be run on ONLY ONE COMPUTER in your network. \
             Scheduler is the main Lifeblood component, responsible for processing all tasks",
            if cfg!(unix) {
                "./lifeblood"
            } else {
                "./lifeblood.cmd"
            },
            vec![],
            Some(vec![
                LaunchControlDataOption::new(
                    "log level",
                    LaunchControlDataOptionValueStorage::new_enum(vec![
                        ("INFO", "info"),
                        ("DEBUG", "debug"),
                    ]),
                    Some("--loglevel"),
                ),
                LaunchControlDataOption::new(
                    "component",
                    LaunchControlDataOptionValueStorage::Nothing,
                    Some("scheduler"),
                ),
                LaunchControlDataOption::new(
                    "pinger's verbosity",
                    LaunchControlDataOptionValueStorage::new_enum(vec![
                        ("INFO", "info"),
                        ("DEBUG", "debug"),
                    ]),
                    Some("--verbosity-pinger"),
                ),
                LaunchControlDataOption::new(
                    "broadcast interval",
                    LaunchControlDataOptionValueStorage::new_int(10),
                    Some("--broadcast-interval"),
                ),
            ]),
        )));
        let wpool_launch_data = Rc::new(RefCell::new(LaunchControlData::new(
            None,
            "Worker Pool",
            "Run this on EVERY computer that needs to do the work",
            if cfg!(unix) {
                "./lifeblood"
            } else {
                "./lifeblood.cmd"
            },
            vec![],
            Some(vec![
                LaunchControlDataOption::new(
                    "log level",
                    LaunchControlDataOptionValueStorage::new_enum(vec![
                        ("INFO", "info"),
                        ("DEBUG", "debug"),
                    ]),
                    Some("--loglevel"),
                ),
                LaunchControlDataOption::new(
                    "component",
                    LaunchControlDataOptionValueStorage::Nothing,
                    Some("pool"),
                ),
                LaunchControlDataOption::new(
                    "component",
                    LaunchControlDataOptionValueStorage::Nothing,
                    Some("simple"),
                ),
            ]),
        )));
        let viewer_launch_data = Rc::new(RefCell::new(LaunchControlData::new(
            None,
            "Viewer",
            "Viewer is a UI to access scheduler over network. You can use it to set up task workflows and monitor task progression",
            if cfg!(unix) {
                "./lifeblood_viewer"
            } else {
                "./lifeblood_viewer.cmd"
            },
            vec![],
            Some(vec![LaunchControlDataOption::new(
                "log level",
                LaunchControlDataOptionValueStorage::new_enum(vec![
                    ("INFO", "info"),
                    ("DEBUG", "debug"),
                ]),
                Some("--loglevel"),
            )]),
        )));

        // main launch widget
        let mut widget = LaunchWidget {
            launch_datas: vec![
                scheduler_launch_data.clone(),
                wpool_launch_data.clone(),
                viewer_launch_data.clone(),
            ],
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
        let mut flex = Flex::default_fill().row();
        flex.set_frame(fltk::enums::FrameType::RShadowBox);
        let label_size = 26;
        let main_margin = 8;
        let margin = 2; // a guess
        flex.set_margin(main_margin);
        let mut group_height = 2 * main_margin + label_size + ITEM_HEIGHT + 4 * margin;

        let mut button_box = Flex::default_fill().column();

        // name and running status
        let heading_group = Flex::default_fill().row();
        Frame::default()
            .with_label(&control_data.borrow().command_label())
            .set_label_size(label_size);
        let mut status_label = Frame::default_fill().with_label("off");
        heading_group.end();
        button_box.fixed(&heading_group, label_size);

        // control options
        let mut options_widgets: Vec<Box<dyn WidgetExt>> = Vec::new();
        for (opt_idx, option) in control_data.borrow().args_options().iter().enumerate() {
            use LaunchControlDataOptionValueStorage::*;
            if let Nothing = option.value() {
                continue;
            }
            group_height += ITEM_HEIGHT + 2 * margin;
            let option_group = Flex::default_fill().row();
            options_widgets.push(Box::new(Frame::default_fill().with_label(option.label())));

            macro_rules! cdgetter {
                ($c:ident) => {
                    if let Some(x) = $c.upgrade() {
                        x
                    } else {
                        println!("[WARNING] callback ui called after data is dropped, ignoring");
                        return;
                    }
                };
            }

            match option.value() {
                RawString(s) => {
                    let mut inp = Input::default_fill();
                    inp.set_value(s);
                    inp.set_trigger(CallbackTrigger::Changed);
                    let control_data = Rc::downgrade(&control_data);
                    inp.set_callback(move |wgt| {
                        let control_data = cdgetter!(control_data);
                        let mut cdata = control_data.borrow_mut();
                        let option = &mut cdata.args_options_mut()[opt_idx];
                        option.set_value(crate::launch_data::LaunchControlDataOptionValue::String(
                            wgt.value(),
                        ));
                    });
                    options_widgets.push(Box::new(inp));
                }
                Int(i) => {
                    let mut inp = IntInput::default_fill();
                    inp.set_value(&i.to_string());
                    inp.set_trigger(CallbackTrigger::Changed);
                    let control_data = Rc::downgrade(&control_data);
                    inp.set_callback(move |wgt| {
                        let control_data = cdgetter!(control_data);
                        let mut cdata = control_data.borrow_mut();
                        let option = &mut cdata.args_options_mut()[opt_idx];
                        option.set_value(crate::launch_data::LaunchControlDataOptionValue::Int(
                            wgt.value().parse().unwrap_or(10),
                        ));
                    });
                    options_widgets.push(Box::new(inp));
                }
                Enum((val_pairs, selected)) => {
                    let mut inp = Choice::default_fill();
                    for (_, label) in val_pairs {
                        inp.add_choice(label);
                    }
                    inp.set_value(*selected as i32);
                    let control_data = Rc::downgrade(&control_data);
                    inp.set_callback(move |wgt| {
                        let control_data = cdgetter!(control_data);
                        let mut cdata = control_data.borrow_mut();
                        let option = &mut cdata.args_options_mut()[opt_idx];
                        option.set_value(crate::launch_data::LaunchControlDataOptionValue::Enum(
                            wgt.value() as usize,
                        ));
                    });
                    options_widgets.push(Box::new(inp));
                }
                Nothing => (),
            }
            option_group.end();
            button_box.fixed(&option_group, ITEM_HEIGHT);
        }
        // launch buttons
        let button_group = Flex::default_fill().row();
        let mut start_button = Button::default_fill().with_label("start");
        let mut stop_button = Button::default_fill().with_label("stop");
        button_group.end();
        button_box.fixed(&button_group, ITEM_HEIGHT);
        button_box.end();

        let info_box = Flex::default_fill().column();
        let pid_label = Frame::default().with_label("not running");
        Frame::default()
            .with_label(control_data.borrow().description())
            .set_align(Align::Left | Align::Inside | Align::Wrap);
        let mut info_label1 = Flex::default_fill().row();
        info_label1.fixed(&Frame::default().with_label("base:"), 48);
        let info_label_running_root = Frame::default().with_label("");
        info_label1.end();
        info_box.end();

        parent_group.fixed(&flex, group_height);

        flex.end();

        let options_widgets_rc = Rc::new(RefCell::new(options_widgets));

        // init state
        stop_button.deactivate();
        if !control_data.borrow_mut().is_current_installation_set() {
            start_button.deactivate();
        }

        // ui callbacks
        let control_data_ref = Rc::downgrade(&control_data);
        let mut start_button_cl = start_button.clone();
        let mut stop_button_cl = stop_button.clone();
        let mut status_label_cl = status_label.clone();
        let mut pid_label_cl = pid_label.clone();
        let mut options_widgets_cl = options_widgets_rc.clone();
        let mut info_label_running_root_cl = info_label_running_root.clone();
        app::add_timeout3(1.0, move |handle| {
            let control_data_ref = if let Some(x) = control_data_ref.upgrade() {
                x
            } else {
                println!("[WARNING] callback ui called after data is dropped, ignoring");
                return;
            };
            let mut data = control_data_ref.borrow_mut();
            if !data.is_process_running() {
                app::repeat_timeout3(2.0, handle);
                return;
            };

            match data.try_wait() {
                Ok(Some(status)) => {
                    let exit_code = status.code().unwrap_or(-1); // read code() help to see why we rewrap this option

                    let status_text = match exit_code {
                        0 => "⚪ finished OK",
                        -1 => "🔴 unhandled signal",
                        1 => "🔴 generic error",
                        2 => "🔴 argument error",
                        x => &format!("🔴 error code: {}", x),
                    };
                    status_label_cl.set_label(status_text);
                    status_label_cl.set_tooltip(status_text);
                    start_button_cl.activate();
                    stop_button_cl.deactivate();
                    Self::change_active_status_on_vec(&mut options_widgets_cl, true);
                    info_label_running_root_cl.set_label("");
                    pid_label_cl.set_label("not running");
                }
                Err(e) => {
                    eprintln!("failed to check process status: {:?}, ignoring", e);
                }
                Ok(None) => {} // we just wait
            };

            app::repeat_timeout3(1.0, handle);
        });

        let control_data_ref = Rc::downgrade(&control_data);
        let mut start_button_cl = start_button.clone();
        let mut stop_button_cl = stop_button.clone();
        let mut status_label_cl = status_label.clone();
        let mut pid_label_cl = pid_label.clone();
        let mut options_widgets_cl = options_widgets_rc.clone();
        let mut info_label_running_root_cl = info_label_running_root.clone();
        start_button.set_callback(move |_| {
            let control_data_ref = if let Some(x) = control_data_ref.upgrade() {
                x
            } else {
                println!("[WARNING] callback ui called after data is dropped, ignoring");
                return;
            };
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
                    status_label_cl.set_tooltip(&err);
                    return;
                }
            };

            start_button_cl.deactivate();
            Self::change_active_status_on_vec(&mut options_widgets_cl, false);
            stop_button_cl.activate();
            status_label_cl.set_label("🟢 running");
            status_label_cl.set_tooltip("running");
            pid_label_cl.set_label(&format!(
                "pid: {}",
                if let Some(pid) = data.process_pid() {
                    pid
                } else {
                    0
                }
            ));
        });

        let control_data_ref = Rc::downgrade(&control_data);
        let mut status_label_cl = status_label.clone();
        let mut stop_button_cl = stop_button.clone();
        stop_button.set_callback(move |_| {
            let control_data_ref = if let Some(x) = control_data_ref.upgrade() {
                x
            } else {
                println!("[WARNING] callback ui called after data is dropped, ignoring");
                return;
            };
            let data = control_data_ref.borrow_mut();
            if let Some(ref proc) = data.process() {
                if let Err(e) = proc.send_terminate_signal() {
                    eprintln!("failed to call terminate on child process cuz of: {:?}", e);
                    return;
                }
                status_label_cl.set_label("🟠 terminating");
                status_label_cl.set_tooltip("terminating...");
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
                            status_label.set_tooltip("installation location is not set or invalid");
                        }
                    } else {
                        if data.is_current_installation_set() {
                            start_button.activate();
                            stop_button.deactivate();
                            status_label.set_label("⚪ ready");
                            status_label.set_tooltip("ready to launch");
                        }
                    }
                },
            )));
    }

    fn change_active_status_on_vec(
        widgets: &mut Rc<RefCell<Vec<Box<dyn WidgetExt>>>>,
        active: bool,
    ) {
        for wgt in widgets.borrow_mut().iter_mut() {
            if active {
                wgt.activate();
            } else {
                wgt.deactivate();
            }
        }
    }
}
