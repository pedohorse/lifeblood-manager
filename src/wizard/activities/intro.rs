use super::super::wizard_activity::WizardActivityTrait;
use fltk::enums::Align;
use fltk::group::Flex;
use fltk::image::PngImage;
use fltk::misc::HelpView;
use fltk::{frame::Frame, prelude::*};

static ICON_DATA: &'static [u8] = include_bytes!("images/intro_noBG.png");

pub struct IntroActivity {}

impl WizardActivityTrait for IntroActivity {
    fn start_activity(&mut self) {
        let mut layout = Flex::default().row();
        let mut icon = Frame::default();
        icon.set_image(Some(PngImage::from_data(ICON_DATA).unwrap()));
        layout.fixed(&icon, 128);
        Frame::default()
            .with_align(Align::Inside | Align::Left)
            .with_label(
                "\
        Welcome to the Lifeblood-Manager's initial config setup wizard!\n\
        \n\
        I Am Wubik, and today I will help you with going through some simple\n\
        decisions that will help you with setting up initial config. \n\
        Lifeblood requires minimal configuration, things like where your DCC\n\
        software is installed in order for a worker to be able to run it.\n\
        ",
            );
        layout.end();

        let mut help_view = HelpView::default();
        // links below are split to prevent trivial bot scraping
        help_view.set_value(concat!("\
        <h2>Reources</h2>\
        <ul>\
        <li> Official <a href=\"https://pedohorse.github.io/lifeblood/usage.html\">documentation</a>\
        <li> Github <a href=\"https://github.com/pedohorse/lifeblood\">repository</a>\
        <li> Telegram support <a href=\"htt", "ps://t.m", "e/+mnkb", "gRxaBYZkODRi\">group</a>\
        <li> Telegram announcements <a href=\"ht", "tps://t.", "me/+jNyVG", "rmHUac4OTYy\">channel</a>\
        </ul>\
        "));
    }

    fn contents_size(&self) -> (i32, i32) {
        (650, 400)
    }

    fn validate(&self) -> Result<(), &str> {
        Ok(())
    }
}

impl IntroActivity {
    pub fn new() -> Self {
        IntroActivity {}
    }
}
