use std::cmp::max;
use std::{cell::RefCell, rc::Rc};

use fltk::app::screen_size;
use fltk::draw::measure;
use fltk::{
    app::wait,
    button::Button,
    frame::Frame,
    group::Flex,
    prelude::{GroupExt, WidgetBase, WidgetExt, WindowExt},
    window::Window,
};

///
/// why? well, default alert and message crash my xwayland
/// probably issues with my setup, mesa, wayland...
/// anyway, this way it works at least
///
pub struct InfoDialog {}

pub struct ChoiceDialog {}

impl InfoDialog {
    pub fn show_in_center(title: &str, text: &str) {
        let (x ,y) = screen_size();

        let (text_w, text_h) = measure(text, false);
        let popup_x = x as i32 / 2 - text_w / 2;
        let popup_y = y as i32 / 2 - 50 - text_h / 2;

        Self::show(popup_x, popup_y, title, text);
    }

    pub fn show(x: i32, y: i32, title: &str, text: &str) {
        let width = max(300, fltk::draw::measure(text, true).0 + 50);

        let mut win = Window::default()
            .with_size(width, 100)
            .with_label(title)
            .with_pos(x - width / 2, y);
        let group = Flex::default_fill().column();
        Frame::default().with_label(text);
        let mut line = Flex::default().row();
        Frame::default();
        let mut btn = Button::default().with_label("Okay");
        btn.set_callback({
            let mut win = win.clone();
            move |_| {
                win.hide();
            }
        });
        line.fixed(&btn, 120);
        line.end();

        group.end();
        win.end();

        win.make_modal(true);
        win.show();
    }
}

impl ChoiceDialog {
    pub fn show(x: i32, y: i32, title: &str, text: &str, choice1: &str, choice2: &str) -> bool {
        let result = Rc::new(RefCell::new(false));

        let width = max(300, fltk::draw::measure(text, true).0 + 50);
        let mut win = Window::default()
            .with_size(width + 50, 100)
            .with_label(title)
            .with_pos(x - width / 2, y);
        let group = Flex::default_fill().column();
        Frame::default().with_label(text);
        let mut line = Flex::default().row();
        Frame::default();
        let mut btn1 = Button::default().with_label(choice1);
        btn1.set_callback({
            let mut win = win.clone();
            let result = result.clone();
            move |_| {
                *result.borrow_mut() = true;
                win.hide();
            }
        });
        let mut btn2 = Button::default().with_label(choice2);
        btn2.set_callback({
            let mut win = win.clone();
            let result = result.clone();
            move |_| {
                *result.borrow_mut() = false;
                win.hide();
            }
        });
        line.fixed(&btn1, 120);
        line.fixed(&btn2, 120);
        line.end();

        group.end();
        win.end();

        win.make_modal(true);
        win.show();

        // block until closed
        while win.shown() {
            wait();
        }

        return *result.borrow_mut();
    }
}
