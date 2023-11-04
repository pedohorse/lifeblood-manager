use fltk::{window::Window, prelude::{WidgetExt, GroupExt, WindowExt, WidgetBase}, group::Flex, app, frame::Frame, button::Button };

///
/// why? well, default alert and message crash my xwayland
/// probably issues with my setup, mesa, wayland...
/// anyway, this way it works at least
///
pub struct InfoDialog{

}

impl InfoDialog {
    pub fn show(x: i32, y: i32, text: &str) {

        let mut win = Window::default().with_size(400, 100).with_label("oh hi there").with_pos(x, y);
        let group = Flex::default_fill().column();
        Frame::default().with_label(text);
        let mut line = Flex::default().row();
        Frame::default();
        let mut btn = Button::default().with_label("Okay");
        btn.set_callback({
            let mut win = win.clone();
            move |_| {
                win.hide();
        }});
        line.fixed(&btn, 120);
        line.end();

        group.end();
        win.end();

        win.make_modal(true);
        win.show();
    }
}