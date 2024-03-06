use fltk::{
    app, button::Button, dialog::NativeFileChooser, frame::Frame, group::Flex, group::Tabs,
    input::FileInput, prelude::*, window::Window,
};
use lifeblood_manager::{
    theme::*, InstallationWidget, InstallationsData, LaunchWidget, StandardEnvResolverConfigWidget,
    Widget, WidgetCallbacks,
};
use std::env::current_dir;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct MainWidget {
    base_path_input: FileInput,
    sub_widgets: Vec<Arc<Mutex<dyn WidgetCallbacks>>>,
    install_data: Option<Arc<Mutex<InstallationsData>>>,
}

impl MainWidget {
    /// interface initialization helpers
    fn init_base_path_input() -> (Button, FileInput, Flex) {
        let mut base_input_flex = Flex::default().row();
        base_input_flex.fixed(&Frame::default().with_label("base directory"), 120);
        let base_input = FileInput::default();
        let browse_button = Button::default().with_label("browse");
        base_input_flex.fixed(&browse_button, 64);
        base_input_flex.end();

        (browse_button, base_input, base_input_flex)
    }

    pub fn new(path: &PathBuf) -> Arc<Mutex<Self>> {
        let mut flex = Flex::default_fill().column();
        // one shared install location
        // base path input
        let (mut browse_button, base_input, base_input_flex) = Self::init_base_path_input();
        flex.fixed(&base_input_flex, ITEM_HEIGHT);

        let path_warning_label = Frame::default().with_label("");
        flex.fixed(&path_warning_label, ITEM_HEIGHT);
        //
        let mut widgets: Vec<Arc<Mutex<dyn WidgetCallbacks>>> = Vec::new();

        let tabs = Tabs::default_fill(); //.with_size(128, 111);
        let (install_widget, _) = InstallationWidget::initialize();
        let (launch_widget, tab_header_flex) = LaunchWidget::initialize();
        let (env_widget, _) = StandardEnvResolverConfigWidget::initialize();

        tabs.end();
        tabs.resizable(&tab_header_flex);
        for c in tabs.clone().into_iter() {
            if let Some(mut c) = c.as_group() {
                c.resize(tabs.x(), tabs.y() + 30, tabs.w(), tabs.h() - 30);
            }
        }

        widgets.push(install_widget);
        widgets.push(launch_widget);
        widgets.push(env_widget);

        flex.end();

        let widget = Arc::new(Mutex::new(MainWidget {
            base_path_input: base_input,
            sub_widgets: widgets,
            install_data: None,
        }));

        // callbacks

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
                    .change_install_dir(&PathBuf::from(input.value()));
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
                widget_to_cb.lock().unwrap().change_install_dir(&input_path);
            }
        });

        // lastly, initialize
        widget.lock().unwrap().change_install_dir(path);

        widget
    }

    pub fn change_install_dir(&mut self, new_path: &PathBuf) {
        // update input
        self.install_data = match InstallationsData::from_dir(new_path.clone()) {
            Ok(x) => Some(Arc::new(Mutex::new(x))),
            _ => {
                println!("no versions found");
                None
            }
        };
        self.base_path_input.set_value(&new_path.to_string_lossy());
        for widget_to_cb in self.sub_widgets.iter_mut() {
            widget_to_cb
                .lock()
                .unwrap()
                .install_location_changed(&new_path, self.install_data.as_ref());
        }
    }
}

fn main() {
    let current_dir = if let Ok(d) = current_dir() {
        d
    } else {
        panic!("failed to get current dir!");
    };

    if let Some(qt_scale_factor) = option_env!("QT_SCALE_FACTOR") {
        let qt_scale_factor = qt_scale_factor.parse::<f32>().unwrap_or(1.0);
        app::Screen::all_screens()
            .iter_mut()
            .for_each(|screen| screen.set_scale(screen.scale() * qt_scale_factor));
    }

    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    app::set_background_color(BG_COLOR[0], BG_COLOR[1], BG_COLOR[2]);
    app::set_foreground_color(FG_COLOR[0], FG_COLOR[1], FG_COLOR[2]);
    app::set_background2_color(BG2_COLOR[0], BG2_COLOR[1], BG2_COLOR[2]);
    app::set_selection_color(SEL_COLOR[0], SEL_COLOR[1], SEL_COLOR[2]);
    app::set_visible_focus(false);

    let mut wind = Window::default()
        .with_size(650, 500)
        .with_label("Lifeblood Manager");

    MainWidget::new(&current_dir);

    wind.end();

    wind.make_resizable(true);
    wind.show();
    app.run().unwrap();
    app::delete_widget(wind); // deleting widgets delets lambdas holding arcs to self

    // // Theming
    // wind.set_color(Color::White);
    // but_inc.set_color(Color::from_u32(0x304FFE));
    // but_inc.set_selection_color(Color::Green);
    // but_inc.set_label_size(20);
    // but_inc.set_frame(FrameType::FlatBox);
    // but_inc.set_label_color(Color::White);
    // but_dec.set_color(Color::from_u32(0x2962FF));
    // but_dec.set_selection_color(Color::Red);
    // but_dec.set_frame(FrameType::FlatBox);
    // but_dec.set_label_size(20);
    // but_dec.set_label_color(Color::White);
    // // End theming
}
