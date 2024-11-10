use super::wizard_activity::{ActivityResult, WizardActivityTrait};
use crate::info_dialog::InfoDialog;
use crate::theme::ITEM_HEIGHT;
use fltk::button::Button;
use fltk::frame::Frame;
use fltk::group::Flex;
use fltk::prelude::*;
use fltk::window::Window;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ActivityRunner {}

struct ActivityWidget<'a> {
    activity: &'a mut dyn WizardActivityTrait,
    result: Rc<RefCell<Option<ActivityResult>>>,
    window: Option<Window>,
}

impl<'a> ActivityWidget<'a> {
    fn create_activity_widget(activity: &'a mut dyn WizardActivityTrait) -> Self {
        ActivityWidget {
            activity: activity,
            result: Rc::new(RefCell::new(None)),
            window: None,
        }
    }

    pub fn show(&mut self) {
        let (width, height) = self.activity.contents_size();
        let mut wind = Window::default()
            .with_size(width, height)
            .with_label("Config Wizard");
        wind.make_modal(true);
        let mut main_layout = Flex::default_fill().column();
        main_layout.set_margin(16);

        self.activity.start_activity();

        let button_row = Flex::default().row();
        main_layout.fixed(&button_row, ITEM_HEIGHT);
        {
            let result = self.result.clone();
            let mut wind = wind.clone();
            Button::default()
                .with_label("@< back")
                .set_callback(move |_| {
                    *result.borrow_mut() = Some(ActivityResult::Prev);
                    wind.hide();
                });
        }

        Frame::default();
        Frame::default();
        
        Button::default().with_label("next @>").set_callback({
            let result = self.result.clone();

            move |_| {
                *result.borrow_mut() = Some(ActivityResult::Next);
            }
        });
        
        button_row.end();

        main_layout.end();
        wind.end();
        
        wind.set_callback({
            let result = self.result.clone();
            move |_| {
                *result.borrow_mut() = Some(ActivityResult::Abort);
            }
        });
        

        wind.show();

        self.window = Some(wind);
    }

    fn hide(&mut self) {
        if let Some(ref mut wind) = self.window {
            wind.hide();
        }
    }

    fn result(&self) -> Option<ActivityResult> {
        return *self.result.borrow();
    }

    fn reset_result(&mut self) {
        *self.result.borrow_mut() = None;
    }
}

impl ActivityRunner {
    pub fn new() -> Self {
        ActivityRunner {}
    }

    pub fn process(&mut self, activity: &mut dyn WizardActivityTrait) -> ActivityResult {
        let mut widget = ActivityWidget::create_activity_widget(activity);
        widget.show();

        loop {
            match widget.result() {
                None => {
                    if let Err(_) = fltk::app::wait_for(0.01) {
                        // according to docs - error means signal interruption, so we can safely ignore it.
                    }
                }
                Some(res) => {
                    // validate
                    if let ActivityResult::Next = res {
                        if let Err(description) = widget.activity.validate() {
                            InfoDialog::show_in_center("data not valid", description);
                            widget.reset_result();
                            continue;
                        }
                    }
                    widget.hide();
                    return res;
                }
            };
        }
    }
}
