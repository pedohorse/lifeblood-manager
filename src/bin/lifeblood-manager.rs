use fltk::{app, group::Tabs, prelude::*, window::Window};
use lifeblood_manager::{InstallationWidget, Widget, theme::*};
use std::env::current_dir;

fn main() {
    let current_dir = if let Ok(d) = current_dir() {
        d
    } else {
        panic!("failed to get current dir!");
    };

    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    app::set_background_color(BG_COLOR[0], BG_COLOR[1], BG_COLOR[2]);
    app::set_foreground_color(FG_COLOR[0], FG_COLOR[1], FG_COLOR[2]);
    app::set_background2_color(BG2_COLOR[0], BG2_COLOR[1], BG2_COLOR[2]);
    app::set_selection_color(SEL_COLOR[0], SEL_COLOR[1], SEL_COLOR[2]);
    app::set_visible_focus(false);

    let mut wind = Window::default().with_size(650, 400).with_label("Lifeblood Manager");
    let mut tabs = Tabs::default_fill();

    let install_widget = InstallationWidget::initialize();
    install_widget
        .lock()
        .unwrap()
        .change_install_dir(current_dir)
        .unwrap_or_else(|_| {
            println!("no versions found in cwd");
        });

    tabs.end();
    tabs.auto_layout();
    wind.end();
    wind.make_resizable(true);
    wind.show();
    app.run().unwrap();

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
