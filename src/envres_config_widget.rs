use crate::config_data::{ConfigData, ConfigError, ConfigWritingError};
use crate::config_data_collection::ConfigDataCollection;
use crate::info_dialog::{ChoiceDialog, InfoDialog};
use crate::theme::ITEM_HEIGHT;
use crate::widgets::{Widget, WidgetCallbacks};
use crate::InstallationsData;
use fltk::button::Button;
use fltk::enums::CallbackTrigger;
use fltk::frame::Frame;
use fltk::group::Flex;
use fltk::input::FileInput;
use fltk::text::{TextBuffer, TextEditor};
use fltk::{app, prelude::*};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct StandardEnvResolverConfigWidget {
    pub config_data: ConfigData,
    has_unsaved_changes: bool,
}

impl Widget for StandardEnvResolverConfigWidget {
    fn initialize() -> (Arc<Mutex<Self>>, Flex) {
        let tab_header = Flex::default_fill()
            .with_label("Environment Resolver Config\t")
            .row();
        let tab_body = Flex::default().column();
        let mut main_layout = Flex::default().column();

        let mut path_row_layout = Flex::default().row();
        path_row_layout.fixed(&Frame::default().with_label("config location"), 128);
        let mut config_path = FileInput::default();
        path_row_layout.end();
        main_layout.fixed(&path_row_layout, ITEM_HEIGHT);

        let config_location = ConfigDataCollection::default_config_location();
        config_path.set_value(&config_location.to_string_lossy());
        config_path.deactivate();

        let buttons_layout = Flex::default().row();
        let mut save_button = Button::default().with_label("save");
        let mut reload_button = Button::default().with_label("reload");
        Frame::default();
        let mut package_template_button = Button::default().with_label("append package template");
        buttons_layout.end();
        main_layout.fixed(&buttons_layout, ITEM_HEIGHT);

        let mut unsaved_label = Frame::default()
            .with_label("has unsaved changes")
            .with_id("ser_config_unsaved_changes_label");
        unsaved_label.hide();
        main_layout.fixed(&unsaved_label, ITEM_HEIGHT / 2);

        let mut config_editor = TextEditor::default().with_id("ser_config_editor");
        config_editor.set_trigger(CallbackTrigger::Changed);

        main_layout.end();
        tab_body.end();
        tab_header.end();

        //tab_body.fixed(&main_layout, 2*ITEM_HEIGHT);

        // make data and widget
        let config_collection = ConfigDataCollection::new(&config_location);
        let mut widget = StandardEnvResolverConfigWidget {
            config_data: config_collection.get_config_data("standard_environment_resolver"),
            has_unsaved_changes: false,
        };
        widget.reload_config();

        let widget_arc = Arc::new(Mutex::new(widget));

        // set callbacks
        reload_button.set_callback({
            let widget_arc_clone = widget_arc.clone();
            move |_| {
                let mut wgt = widget_arc_clone.lock().unwrap();
                wgt.reload_config();
            }
        });
        save_button.set_callback({
            let widget_arc_clone = widget_arc.clone();
            move |_| {
                let mut wgt = widget_arc_clone.lock().unwrap();
                wgt.save_config();
            }
        });
        package_template_button.set_callback({
            let widget_arc_clone = widget_arc.clone();
            move |_| {
                let mut wgt = widget_arc_clone.lock().unwrap();
                wgt.append_new_package_template_section();
            }
        });
        config_editor.set_callback({
            let widget_arc_clone = widget_arc.clone();
            move |_| {
                let mut wgt = widget_arc_clone.lock().unwrap();
                wgt.mark_as_has_unsaved_changes();
            }
        });

        (widget_arc, tab_header)
    }
}

impl WidgetCallbacks for StandardEnvResolverConfigWidget {
    fn install_location_changed(
        &mut self,
        _path: &PathBuf,
        _install_data: Option<&Arc<Mutex<InstallationsData>>>,
    ) {
        // do nothing
    }

    fn on_tab_selected(&mut self) {
        if !self.has_unsaved_changes {
            self.reload_config();
        }
    }
}

impl StandardEnvResolverConfigWidget {
    pub fn mark_as_has_unsaved_changes(&mut self) {
        self.has_unsaved_changes = true;
        let mut label: Frame = app::widget_from_id("ser_config_unsaved_changes_label").unwrap();
        label.show();
        Flex::from_dyn_widget(&label.parent().unwrap())
            .unwrap()
            .layout();
    }

    fn clear_unsaved_changes(&mut self) {
        self.has_unsaved_changes = false;
        let mut label: Frame = app::widget_from_id("ser_config_unsaved_changes_label").unwrap();
        label.hide();
        Flex::from_dyn_widget(&label.parent().unwrap())
            .unwrap()
            .layout();
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.has_unsaved_changes
    }

    // callbacks
    fn reload_config(&mut self) {
        let mut config_editor: TextEditor = app::widget_from_id("ser_config_editor").unwrap();

        if self.has_unsaved_changes {
            let wind = config_editor.window().unwrap();
            let popup_x = wind.x() + wind.w() / 2 - 100;
            let popup_y = wind.y() + wind.h() / 2 - 50;

            let proceed = ChoiceDialog::show(
                popup_x,
                popup_y,
                "unsaved changes",
                "discard unsaved changes?",
                "yes",
                "no",
            );

            if !proceed {
                return;
            }
        }

        let mut buf = if let Some(buf) = config_editor.buffer() {
            buf
        } else {
            let buf = TextBuffer::default();
            config_editor.set_buffer(buf);
            config_editor.buffer().unwrap()
        };

        buf.set_text(&self.config_data.main_config_text());
        self.clear_unsaved_changes();
        buf.unhighlight();
    }

    fn save_config(&mut self) {
        let config_editor: TextEditor = app::widget_from_id("ser_config_editor").unwrap();

        if let Some(mut buf) = config_editor.buffer() {
            match self.config_data.set_main_config_text(&buf.text()) {
                Err(ConfigWritingError::IoError(e)) => eprintln!("ERROR writing config: {:?}", e),
                Err(ConfigWritingError::ConfigError(e)) => {
                    let wind = config_editor.window().unwrap();
                    let popup_x = wind.x() + wind.w() / 2 - 100;
                    let popup_y = wind.y() + wind.h() / 2 - 50;
                    InfoDialog::show(
                        popup_x,
                        popup_y,
                        "config validation error",
                        &format!("Config is invalid: {:?}", e),
                    );
                    if let ConfigError::SyntaxError(_, Some(span)) = e {
                        buf.highlight(span.start as i32, span.end as i32);
                    }
                }
                Ok(_) => {
                    println!("successfully written config file");
                    self.clear_unsaved_changes();
                    buf.unhighlight();
                }
            }
        } else {
            // hmm ...
            return;
        }
    }

    fn append_new_package_template_section(&mut self) {
        let mut config_editor: TextEditor = app::widget_from_id("ser_config_editor").unwrap();

        let mut buf = if let Some(buf) = config_editor.buffer() {
            buf
        } else {
            let buf = TextBuffer::default();
            config_editor.set_buffer(buf);
            config_editor.buffer().unwrap()
        };

        buf.append(
            r#"
[packages."package_name_here"."1.0.0"]
label = "add package description label here"
env.PATH.prepend = [
    "/path/to/bin",
]
"#,
        );

        self.mark_as_has_unsaved_changes();
    }
}
