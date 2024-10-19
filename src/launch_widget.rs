use crate::launch_data::{
    LaunchControlData, LaunchControlDataOption, LaunchControlDataOptionValueStorage,
};
use crate::theme::ITEM_HEIGHT;
use crate::tray_manager::{TrayItemHandle, TrayManager};
use crate::widgets::{Widget, WidgetCallbacks};
use crate::InstallationsData;
use crate::MainWidgetConfig;
use fltk::button::{Button, CheckButton};
use fltk::enums::{Align, CallbackTrigger};
use fltk::input::{Input, IntInput};
use fltk::menu::Choice;
use fltk::{app, frame::Frame, group::Flex, prelude::*};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
#[cfg(windows)]
use winconsole::window;
#[cfg(windows)]
use crate::win_console_hack::is_console;

pub struct LaunchWidget {
    config: Rc<RefCell<MainWidgetConfig>>,
    launch_datas: Vec<Rc<RefCell<LaunchControlData>>>,
    launches: HashMap<String, (Box<dyn FnMut() -> ()>, Box<dyn FnMut() -> ()>, Box<dyn Fn() -> bool>)>,
    tray_item_handlers: Rc<RefCell<HashMap<String, TrayItemHandle>>>,
}

impl WidgetCallbacks for LaunchWidget {
    fn post_initialize(&mut self) {
        // do autostart if needed
        {
            let config = self.config.clone();
            for launch_id in config.borrow().launch_ids_to_autostart() {
                if let Err(_e) = self.start_process_by_id(launch_id) {
                    eprintln!("failed to autostart '{}'", launch_id);
                };
            }
        }

    }

    fn install_location_changed(
        &mut self,
        _path: &Path,
        install_data: Option<&Arc<Mutex<InstallationsData>>>,
    ) {
        for launch_data in self.launch_datas.iter_mut() {
            launch_data
                .borrow_mut()
                .install_location_changed(install_data);
        }
    }

    fn generate_tray_items(&mut self, tray_manager: &mut TrayManager) {
        for launch_data in self.launch_datas.iter() {
            let launch_id = launch_data.borrow().launch_id().to_owned();
            let (_starter, _stopper, is_running) = if let Some(x) = self.launches.get(&launch_id) {
                x
            } else {
                eprintln!("no widget for launch '{}'", &launch_id);
                continue;
            };

            let status = if is_running() { "running" } else { "stopped" };
            match tray_manager.add_tray_item(&format!("{}: {}", &launch_id, status), |_| {
                // TODO: implement clicking the tray itme
            }) {
                Ok(handle) => {
                    self.tray_item_handlers
                        .borrow_mut()
                        .insert(launch_id, handle);
                }
                Err(_) => {
                    eprintln!("failed to generate tray item for {}", &launch_id);
                }
            };
        }
    }

    fn on_tab_selected(&mut self) {}
}

impl Widget for LaunchWidget {
    fn initialize(config: Rc<RefCell<MainWidgetConfig>>) -> (Arc<Mutex<Self>>, Flex) {
        let tab_header = Flex::default_fill().with_label("Launch\t").row();
        let mut flex = Flex::default_fill().column();
        flex.set_margin(8);
        flex.set_spacing(16);

        // different launch options
        let scheduler_launch_data = Rc::new(RefCell::new(LaunchControlData::new(
            "scheduler",
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
            true,
        )));
        let wpool_launch_data = Rc::new(RefCell::new(LaunchControlData::new(
            "worker pool",
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
            true,
        )));
        let viewer_launch_data = Rc::new(RefCell::new(LaunchControlData::new(
            "viewer",
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
            false,
        )));

        // main launch widget
        let mut widget = LaunchWidget {
            config: config.clone(),
            launch_datas: vec![
                scheduler_launch_data.clone(),
                wpool_launch_data.clone(),
                viewer_launch_data.clone(),
            ],
            launches: HashMap::new(),
            tray_item_handlers: Rc::new(RefCell::new(HashMap::new())),
        };
        widget.make_launch_buttons(&mut flex, scheduler_launch_data);
        widget.make_launch_buttons(&mut flex, wpool_launch_data);
        widget.make_launch_buttons(&mut flex, viewer_launch_data);

        // for windows - generate buttons to show/hide root console with logs
        #[cfg(windows)]
        if !is_console() {
            let horizontal_flex = Flex::default_fill().row();
            let mut show_btn = Button::default().with_label("show root console");
            let mut hide_btn = Button::default().with_label("hide root console");
            show_btn.set_callback(|_| {
                window::activate(true);
            });
            hide_btn.set_callback(|_| {
                window::hide();
            });
            horizontal_flex.end();
            flex.fixed(&horizontal_flex, ITEM_HEIGHT);
        }


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
        if control_data.borrow().allow_autostart() {
            let control_data = control_data.borrow();
            let mut autostart_checkbox = CheckButton::default().with_label("start automatically");
            autostart_checkbox.set_checked(self.config.borrow().has_autostart_launch_id(&control_data.launch_id()));

            autostart_checkbox.set_callback({
                let config = self.config.clone();
                let launch_id = control_data.launch_id().to_string();
                move |wgt| {
                    let mut config = config.borrow_mut();
                    if wgt.is_checked() {
                        config.add_autostart_launch_id(&launch_id);
                    } else {
                        config.remove_autostart_launch_id(&launch_id);
                    }
                    if let Err(e) = config.write_to_file() {
                        eprintln!("failed to save config file: {}", e);
                    }
                }
            });
        }

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
        app::add_timeout3(1.0, {
            let control_data_ref = Rc::downgrade(&control_data);
            let mut start_button_cl = start_button.clone();
            let mut stop_button_cl = stop_button.clone();
            let mut status_label_cl = status_label.clone();
            let mut pid_label_cl = pid_label.clone();
            let mut options_widgets_cl = options_widgets_rc.clone();
            let mut info_label_running_root_cl = info_label_running_root.clone();
            let tray_item_handlers = self.tray_item_handlers.clone();
            move |handle| {
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
                        if let Some(x) = tray_item_handlers.borrow_mut().get_mut(data.launch_id()) {
                            if let Err(_) = x.change_label(&format!("{}: stopped", data.launch_id())) {
                                eprintln!("failed to change tray menu item label for {}", data.launch_id());
                            }
                        }
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
            }
        });

        let mut callback_start = {
            let control_data_ref = Rc::downgrade(&control_data);
            let mut start_button_cl = start_button.clone();
            let mut stop_button_cl = stop_button.clone();
            let mut status_label_cl = status_label.clone();
            let mut pid_label_cl = pid_label.clone();
            let mut options_widgets_cl = options_widgets_rc.clone();
            let mut info_label_running_root_cl = info_label_running_root.clone();
            let tray_item_handlers = self.tray_item_handlers.clone();
            move || {
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

                if let Some(x) = tray_item_handlers.borrow_mut().get_mut(data.launch_id()) {
                    if let Err(_) = x.change_label(&format!("{}: running", data.launch_id())) {
                        eprintln!("failed to change tray menu item label for {}", data.launch_id());
                    }
                }
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
            }
        };

        let mut callback_stop = {
            let control_data_ref = Rc::downgrade(&control_data);
            let mut status_label_cl = status_label.clone();
            let mut stop_button_cl = stop_button.clone();
            let tray_item_handlers = self.tray_item_handlers.clone();
            move || {
                let control_data_ref = if let Some(x) = control_data_ref.upgrade() {
                    x
                } else {
                    println!("[WARNING] callback ui called after data is dropped, ignoring");
                    return;
                };
                let data = control_data_ref.borrow_mut();
                if !data.is_process_running() {
                    return;
                };
                if let Some(ref proc) = data.process() {
                    if let Err(e) = proc.send_terminate_signal() {
                        eprintln!("failed to call terminate on child process cuz of: {:?}", e);
                        status_label_cl.set_tooltip(&format!("failed to call terminate on child process cuz of: {:?}", e));
                        return;
                    }
                    if let Some(x) = tray_item_handlers.borrow_mut().get_mut(data.launch_id()) {
                        if let Err(_) = x.change_label(&format!("{}: terminating...", data.launch_id())) {
                            eprintln!("failed to change tray menu item label for {}", data.launch_id());
                        }
                    }
                    status_label_cl.set_label("🟠 terminating");
                    status_label_cl.set_tooltip("terminating...");
                    stop_button_cl.deactivate();
                };
            }
        };

        let callback_status = {
            let control_data_ref = Rc::downgrade(&control_data);
            move || {
                let control_data_ref = if let Some(x) = control_data_ref.upgrade() {
                    x
                } else {
                    println!("[WARNING] callback ui called after data is dropped, ignoring");
                    return false;
                };
                let data = control_data_ref.borrow_mut();
                data.is_process_running()
            }
        };

        self.launches.insert(
            control_data.borrow().launch_id().to_owned(),
            (
                Box::new(callback_start.clone()),
                Box::new(callback_stop.clone()),
                Box::new(callback_status),
            ),
        );
        start_button.set_callback(move |_| {
            callback_start();
        });
        stop_button.set_callback({
            move |_| {
                callback_stop();
            }
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

    pub fn start_process_by_id(&mut self, id: &str) -> Result<(), ()> {
        if let Some((ref mut starter, _, _)) = self.launches.get_mut(id) {
            Ok(starter())
        } else {
            Err(())
        }
    }

    pub fn stop_process_by_id(&mut self, id: &str) -> Result<(), ()> {
        if let Some((_, ref mut stopper, _)) = self.launches.get_mut(id) {
            Ok(stopper())
        } else {
            Err(())
        }
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
