use crate::widgets::Widget;
use crate::InstallationsData;
use crate::theme::*;
use fltk::dialog;
use fltk::enums::LabelType;
use fltk::{
    app,
    button::Button,
    dialog::NativeFileChooser,
    draw,
    enums::{self, Color, FrameType},
    frame::Frame,
    group::{Flex, Group, Tabs},
    input::FileInput,
    prelude::*,
    table::{Table, TableContext},
};
use std::sync::PoisonError;
use std::{
    path::PathBuf,
    sync::{Mutex, TryLockError},
};
use std::{
    sync::Arc,
    thread,
    time::Duration,
};

const ITEM_HEIGHT: i32 = 32;

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
    install_data: Option<InstallationsData>,
    base_path_input: FileInput,
    installation_table: Table,
    warning_label: Frame,
}

impl InstallationWidget {
    pub fn change_install_dir(&mut self, new_path: PathBuf) -> Result<(), std::io::Error> {
        let new_data = match InstallationsData::from_dir(new_path.clone()) {
            Ok(x) => x,
            Err(e) => {
                self.installation_table.set_rows(0);
                return Err(e);
            }
        };

        // update input
        self.base_path_input.set_value(&new_path.to_string_lossy());

        if new_data.is_base_path_tainted() {
            self.warning_label.set_label(
                "Warning: given path contains elements unrelated to lifeblood.\n\
                       It's recommended to choose an empty directory for lifeblood installations") 
        } else {
            self.warning_label.set_label("");
        }

        self.install_data = Some(new_data);

        // also update table shit
        self.update_installation_table();

        Ok(())
    }

    fn update_installation_table(&mut self) {
        if let Some(data) = &self.install_data {
            self.installation_table
                .set_rows(data.version_count() as i32);
            self.installation_table.redraw();
        }
    }

    /// interface initialization helpers
    fn init_base_path_input() -> (Button, FileInput, Flex) {
        let mut base_input_flex = Flex::default().row();
        base_input_flex.fixed(&Frame::default().with_label("base directory"), 120);
        let mut base_input = FileInput::default();
        let mut browse_button = Button::default().with_label("browse");
        base_input_flex.fixed(&browse_button, 64);
        base_input_flex.end();

        (browse_button, base_input, base_input_flex)
    }
}

impl Widget for InstallationWidget {
    fn initialize() -> Arc<Mutex<Self>> {
        let mut tab_header = Flex::default_fill().with_label("installation\t").row();
        let mut flex = Flex::default_fill().column();
        flex.set_margin(8);
        flex.set_spacing(16);

        // base path input
        let (mut browse_button, base_input, base_input_flex) = Self::init_base_path_input();
        flex.fixed(&base_input_flex, ITEM_HEIGHT);

        let path_warning_label = Frame::default().with_label("");
        flex.fixed(&path_warning_label, ITEM_HEIGHT);

        tab_header.resizable(&tab_header);

        let mut installations_table = Table::default().with_size(200, 200);
        //tab_header.resizable(widget)
        installations_table.set_rows(0);
        installations_table.set_cols(3);
        installations_table.set_col_resize(true);
        installations_table.set_row_resize(true);
        installations_table.set_col_width(0, 64);
        installations_table.set_col_width(1, 300);
        installations_table.set_col_width(2, 150);

        installations_table.end();

        // buttons
        let mut version_control_flex = Flex::default().row();
        flex.fixed(&version_control_flex, ITEM_HEIGHT);
        let mut new_install_btn = Button::default().with_label("download freshest");
        version_control_flex.fixed(&new_install_btn, 150);
        Frame::default();
        let mut make_current_btn = Button::default().with_label("make selected version current");
        version_control_flex.fixed(&make_current_btn, 220);
        version_control_flex.end();

        flex.end();
        tab_header.end();

        let mut widget = InstallationWidget {
            install_data: None,
            base_path_input: base_input,
            installation_table: installations_table,
            warning_label: path_warning_label,
        };

        let widget = Arc::new(Mutex::new(widget));

        //
        // callbacks
        //
        // base path input change callback
        let widget_to_cb = widget.clone();
        widget
            .lock()
            .expect("impossible during init")
            .base_path_input
            .set_callback(move |input| {
                widget_to_cb
                    .lock()
                    .unwrap()
                    .change_install_dir(PathBuf::from(input.value()))
                    .unwrap_or_else(|_| {
                        println!("callback failed to set path");
                    });
            });

        // file dialog chooser callback
        let widget_to_cb = widget.clone();
        browse_button.set_callback(move |_| {
            let mut dialog = NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseDir);
            dialog.show();
            let input_path = dialog.filename();
            let input_str = &input_path.to_string_lossy();
            if input_str != "" {
                //base_input_rc_callback.borrow_mut().set_value(input_str);
                widget_to_cb
                    .lock()
                    .unwrap()
                    .change_install_dir(input_path)
                    .unwrap_or_else(|_| {
                        println!("callback failed to set path");
                    });
            }
        });

        // table draw callback
        let widget_to_cb = widget.clone();
        widget
            .lock()
            .unwrap()
            .installation_table
            .draw_cell(move |t, ctx, row, col, x, y, w, h| {
                let ver_id = (t.rows()-1 - row) as usize;
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
                                        Some(ref data)
                                            if data.current_version_index() == ver_id =>
                                        {
                                            CELL_BG_CUR_COLOR
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
                                if let Some(ref data) = guard.install_data {
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
                                                ver.source_commit(),
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

        // set current button callback
        let widget_to_cb = widget.clone();
        make_current_btn.set_callback(move |_| {
            let mut guard = widget_to_cb.lock().unwrap();

            let (row, _, _, _) = guard.installation_table.get_selection();
            if row < 0 {
                return;
            }
            let ver_id = (guard.installation_table.rows()-1 - row) as usize;
            match guard.install_data {
                Some(ref mut data) => {
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
            thread::scope(|scope| {
                let handle = scope.spawn(|| {
                    let guard = &mut widget_to_cb.lock().unwrap();
                    match guard.install_data {
                        Some(ref mut data) => {
                            // download latest
                            let new_ver = match data.download_new_version() {
                                Ok(idx) => {
                                    // TODO: result process somehow
                                    idx
                                }
                                Err(e) => {
                                    let err_msg = format!("failed to install new version: {}", e);
                                    dialog::alert_default(&err_msg);
                                    eprintln!("{}", err_msg);
                                    return;
                                }
                            };
                            // make current
                            if let Err(e) = data.make_version_current(new_ver) {
                                let err_msg = format!("failed to make new version current: {}", e);
                                dialog::alert_default(&err_msg);
                                eprintln!("{}", err_msg);
                            }
                        }
                        _ => (),
                    }
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
                if let Err(e) = handle.join() {
                    eprintln!("thead join failed! {:?}", e);
                }
            });

            widget_to_cb.lock().unwrap().update_installation_table();
        });

        widget
    }
}
