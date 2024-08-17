use crate::config_data_collection::ConfigDataCollection;
use crate::theme::*;
use crate::widgets::{Widget, WidgetCallbacks};
use crate::InstallationsData;
use crate::info_dialog::InfoDialog;
use crate::installation_helpers::get_python_command;
use crate::wizard::Wizard;
use fltk::button::CheckButton;
use fltk::dialog;
use fltk::misc::InputChoice;
use fltk::{
    app,
    button::Button,
    draw,
    enums,
    frame::Frame,
    group::Flex,
    prelude::*,
    table::{Table, TableContext},
};
use std::sync::MutexGuard;
use std::{
    path::PathBuf,
    sync::{Mutex, TryLockError},
};
use std::{sync::Arc, thread, time::Duration};

const DEFAULT_BRANCH: &str = "dev";

const DOWNLOAD_LABEL_ANIM: [&str; 12] = [
    "working... ( 🕛     )",
    "working... (  🕐    )",
    "working... (   🕑   )",
    "working... (    🕒  )",
    "working... (     🕓 )",
    "working... (      🕔)",
    "working... (     🕕 )",
    "working... (    🕖  )",
    "working... (   🕗   )",
    "working... (  🕘    )",
    "working... ( 🕙     )",
    "working... (🕚      )",
];

pub struct InstallationWidget {
    install_data: Option<Arc<Mutex<InstallationsData>>>,
    installation_table: Table,
    warning_label: Frame,
    main_flex: Flex,
}

fn lock_install_data(data: &Arc<Mutex<InstallationsData>>) -> MutexGuard<'_, InstallationsData> {
    if let Ok(x) = data.lock() { x } else{
        panic!("failed to lock installations data!");
    }
}

impl InstallationWidget {
    pub fn change_install_dir(&mut self, _new_path: &PathBuf, install_data: Option<&Arc<Mutex<InstallationsData>>>) -> Result<(), std::io::Error> {
        let new_data = match install_data {
            Some(x) => x.clone(),
            None => {
                self.installation_table.set_rows(0);
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "path has no lifeblood installations"));
            }
        };
        
        let guard = lock_install_data(&new_data);

        if guard.is_base_path_tainted() {
            self.warning_label.set_label(
                "Warning: given path contains elements unrelated to lifeblood.\n\
                       It's recommended to choose an empty directory for lifeblood installations",
            );
            self.main_flex.fixed(&self.warning_label, ITEM_HEIGHT * 2);
        } else {
            self.warning_label.set_label("");
            self.main_flex.fixed(&self.warning_label, 1);
        }
        drop(guard);

        self.install_data = Some(new_data);

        // also update table shit
        self.update_installation_table();
        self.main_flex.recalc(); // also forces redraw on all children that is needed after label change

        Ok(())
    }

    fn update_installation_table(&mut self) {
        if let Some(mutexed_data) = &self.install_data {
            let data = lock_install_data(&mutexed_data);
            self.installation_table
                .set_rows(data.version_count() as i32);
            self.installation_table.redraw();
        }
    }

}

impl WidgetCallbacks for InstallationWidget {
    fn install_location_changed(&mut self, path: &PathBuf, install_data: Option<&Arc<Mutex<InstallationsData>>>){
        self.change_install_dir(path, install_data).unwrap_or_else(|_| {
            println!("failed to set path to {:?}", path);
        })
    }
}

impl Widget for InstallationWidget {
    fn initialize() -> (Arc<Mutex<Self>>, Flex) {
        let tab_header = Flex::default_fill().with_label("Installation\t").row();
        let mut flex = Flex::default_fill().column();
        flex.set_margin(8);
        flex.set_spacing(16);

        let path_warning_label = Frame::default().with_label("");
        flex.fixed(&path_warning_label, ITEM_HEIGHT);

        tab_header.resizable(&tab_header);

        let mut installations_table = Table::default().with_size(200, 200);
        //tab_header.resizable(widget)
        installations_table.set_rows(0);
        installations_table.set_cols(5);
        installations_table.set_col_resize(true);
        installations_table.set_row_resize(true);
        installations_table.set_col_width(0, 64);
        installations_table.set_col_width(1, 250);
        installations_table.set_col_width(2, 150);
        installations_table.set_col_width(3, 16);
        installations_table.set_col_width(4, 350);

        installations_table.end();

        // buttons
        let mut control_buttons_group_vertical = Flex::default().column();
        flex.fixed(&control_buttons_group_vertical, 2*ITEM_HEIGHT);

        let upper_control_row = Flex::default().row();
        control_buttons_group_vertical.fixed(&upper_control_row, ITEM_HEIGHT);
        let ignore_system_python_checkbox = CheckButton::default().with_label("ignore system python");
        let mut wizard_button = Button::default().with_label("Config Wizard");
        upper_control_row.end();

        let mut version_control_flex = Flex::default().row();
        control_buttons_group_vertical.fixed(&version_control_flex, ITEM_HEIGHT);
        let mut new_install_btn = Button::default().with_label("download freshest");
        version_control_flex.fixed(&new_install_btn, 150);
        let mut branch_selector = InputChoice::default();
        branch_selector.add(DEFAULT_BRANCH);
        branch_selector.set_value(DEFAULT_BRANCH);
        Frame::default();
        let mut rename_ver_btn = Button::default().with_label("rename selected");
        let mut make_current_btn = Button::default().with_label("make selected version current");
        version_control_flex.fixed(&rename_ver_btn, 130);
        version_control_flex.fixed(&make_current_btn, 230);
        version_control_flex.end();

        control_buttons_group_vertical.end();

        flex.end();
        tab_header.end();

        let widget = InstallationWidget {
            install_data: None,
            installation_table: installations_table,
            warning_label: path_warning_label,
            main_flex: flex,
        };

        let widget = Arc::new(Mutex::new(widget));

        //
        // callbacks
        //

        // table draw callback
        let widget_to_cb = widget.clone();
        widget
            .lock()
            .unwrap()
            .installation_table
            .draw_cell(move |t, ctx, row, col, x, y, w, h| {
                let ver_id = (t.rows() - 1 - row) as usize;
                match ctx {
                    TableContext::Cell => {
                        draw::push_clip(x, y, w, h);
                        draw::draw_box(
                            enums::FrameType::ThinDownBox,
                            x,
                            y,
                            w,
                            h,
                            if t.is_selected(row, col) {
                                CELL_BG_SEL_COLOR
                            } else {
                                match &widget_to_cb.try_lock() {
                                    Ok(guard) => match guard.install_data {
                                        Some(ref mutexed_data) =>
                                        {
                                            // TODO: OMG, we are doing 2 locks for EACH CELL !!
                                            let data = lock_install_data(&mutexed_data);
                                            if data.current_version_index() == ver_id {
                                                CELL_BG_CUR_COLOR
                                            } else {
                                                CELL_BG_COLOR
                                            }
                                            
                                        }
                                        _ => CELL_BG_COLOR,
                                    },
                                    _ => CELL_BG_COLOR,
                                }
                            },
                        );
                        draw::set_draw_color(CELL_FG_COLOR);
                        match &widget_to_cb.try_lock() {
                            Ok(guard) => {
                                if let Some(ref mutexed_data) = guard.install_data {
                                    let data = lock_install_data(&mutexed_data);
                                    match data.version(ver_id) {
                                        Some(ver) => match col {
                                            0 => {
                                                if ver_id == data.current_version_index() {
                                                    draw::draw_text2(
                                                        "current",
                                                        x,
                                                        y,
                                                        w,
                                                        h,
                                                        enums::Align::Center,
                                                    );
                                                }
                                            }
                                            1 => draw::draw_text2(
                                                ver.nice_name(),
                                                x,
                                                y,
                                                w,
                                                h,
                                                enums::Align::Center,
                                            ),
                                            2 => draw::draw_text2(
                                                &ver.date().format("%d-%m-%Y %H:%M:%S").to_string(),
                                                x,
                                                y,
                                                w,
                                                h,
                                                enums::Align::Center,
                                            ),
                                            3 => draw::draw_text2(
                                                if ver.has_viewer() { "v" } else { " " },
                                                x,
                                                y,
                                                w,
                                                h,
                                                enums::Align::Center,
                                            ),
                                            4 => draw::draw_text2(
                                                ver.source_commit(),
                                                x,
                                                y,
                                                w,
                                                h,
                                                enums::Align::Center,
                                            ),
                                            _ => draw::draw_text2(
                                                "<ERROR>",
                                                x,
                                                y,
                                                w,
                                                h,
                                                enums::Align::Center,
                                            ),
                                        },
                                        None => {
                                            draw::draw_text2(
                                                "<ERROR>",
                                                x,
                                                y,
                                                w,
                                                h,
                                                enums::Align::Center,
                                            );
                                        }
                                    }
                                }
                            }
                            Err(TryLockError::WouldBlock) => {
                                draw::draw_text2(
                                    "<data update in progress>",
                                    x,
                                    y,
                                    w,
                                    h,
                                    enums::Align::Center,
                                );
                            }
                            _ => {
                                draw::draw_text2("<ALL BROKEN!>", x, y, w, h, enums::Align::Center);
                            }
                        }
                        draw::pop_clip();
                    }
                    _ => (),
                }
            });

        // rename button callback
        let widget_to_cb = widget.clone();
        rename_ver_btn.set_callback(move |btn| {
            let mut guard = widget_to_cb.lock().unwrap();
            let (row, _, _, _) = guard.installation_table.get_selection();
            if row < 0 {
                return;
            }

            let ver_id = (guard.installation_table.rows() - 1 - row) as usize;
            let mut install_data = if let Some(data) = &mut guard.install_data {
                lock_install_data(data)
            } else {
                return;
            };

            let wind = btn.window().unwrap();
            let popup_x = wind.x() + wind.w() / 2 - 100;
            let popup_y = wind.y() + wind.h() / 2 - 50;

            let new_name = if let Some(s) = dialog::input(
                popup_x,
                popup_y,
                "new name",
                if let Some(v) = install_data.version(ver_id) {
                    v.nice_name()
                } else {
                    "new_name"
                },
            ) {
                s
            } else {
                return;
            };
            
            if let Err(e) = install_data.rename_version(ver_id, new_name) {
                eprintln!("failed to rename! {}", e);
                InfoDialog::show(popup_x, popup_y, "error", &format!("failed to rename! {}", e));
            }
            drop(install_data);
            
            guard.installation_table.redraw();
        });

        // set current button callback
        let widget_to_cb = widget.clone();
        make_current_btn.set_callback(move |_| {
            let mut guard = widget_to_cb.lock().unwrap();

            let (row, _, _, _) = guard.installation_table.get_selection();
            if row < 0 {
                return;
            }
            let ver_id = (guard.installation_table.rows() - 1 - row) as usize;
            match guard.install_data {
                Some(ref mut mutexed_data) => {
                    let mut data = lock_install_data(&mutexed_data);
                    data.make_version_current(ver_id).unwrap_or_else(|e| {
                        eprintln!("failed to set current version to {}, cuz: {}", ver_id, e);
                    });
                }
                _ => (),
            }
            guard.installation_table.redraw();
        });

        // download freshhhh
        let widget_to_cb = widget.clone();
        new_install_btn.set_callback(move |btn| {
            let branch = match branch_selector.value() {
                Some(x) => x,
                None => DEFAULT_BRANCH.to_owned(),
            };
            let ignore_system_python = ignore_system_python_checkbox.value();
            let mut installation_succeeded = true;

            thread::scope(|scope| {
                let handle = scope.spawn(|| {
                    let guard = &mut widget_to_cb.lock().unwrap();
                    match guard.install_data {
                        Some(ref mut mutexed_data) => {
                            let mut data = lock_install_data(&mutexed_data);
                            // if checkbox is set - we don't try to locate python
                            println!("initiating new version installation");
                            let path_to_python = if ignore_system_python {
                                None
                            } else {
                                get_python_command()
                            };

                            if let Some(ref path) = path_to_python {
                                println!("using python: {:?}", path);
                            }
                            
                            // download latest
                            let new_ver = match data.download_new_version(&branch, true, path_to_python.as_deref()) {
                                Ok(idx) => {
                                    // TODO: result process somehow
                                    idx
                                }
                                Err(e) => {
                                    let err_msg = format!("failed to install new version: {}", e);
                                    return Err(err_msg);
                                }
                            };
                            // make current
                            if let Err(e) = data.make_version_current(new_ver) {
                                let err_msg = format!("failed to make new version current: {}", e);
                                eprintln!("Warning: {}", err_msg);
                            }
                        }
                        _ => (),
                    }
                    Ok(())
                });

                let btn_text = btn.label();
                let mut anim_frame = 0;
                // poll and keep UI responsive
                while !handle.is_finished() {
                    btn.set_label(DOWNLOAD_LABEL_ANIM[anim_frame]);
                    anim_frame = (anim_frame + 1) % DOWNLOAD_LABEL_ANIM.len();
                    app::check();
                    // app::flush();
                    std::thread::sleep(Duration::from_millis(100));
                }
                btn.set_label(&btn_text);

                // join
                match handle.join() {
                    Ok(Err(err_msg)) => {
                        installation_succeeded = false;
                        eprintln!("{}", err_msg);
                        let wind = btn.window().unwrap();
                        InfoDialog::show(
                            wind.x() + (wind.w() / 2) as i32 - 300,
                            wind.y() + (wind.h() / 2) as i32 - 100,
                            "error",
                            &err_msg,
                        );
                    }
                    Err(e) => {
                        eprintln!("thead join failed! {:?}", e);
                    }
                    _ => (),
                }
            });

            widget_to_cb.lock().unwrap().update_installation_table();

            // if conditions are met - also run wizard
            if installation_succeeded {
                let config_root = ConfigDataCollection::default_config_location();
                if !config_root.exists() {
                    let mut wizard = Wizard::new(config_root);
                    wizard.run();
                }
            }
        });

        // wizard callback
        wizard_button.set_callback(|_| {
            let mut wizard = Wizard::new(ConfigDataCollection::default_config_location());
            wizard.run();
        });

        (widget, tab_header)
    }
}
