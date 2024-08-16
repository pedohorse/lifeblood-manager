use super::super::wizard_activity::WizardActivityTrait;
use super::super::wizard_data::{BlenderVersion, HoudiniVersion};
use fltk::misc::HelpView;
use std::path::Path;

pub struct SummaryActivity {
    text: String,
}

impl WizardActivityTrait for SummaryActivity {
    fn start_activity(&mut self) {
        let mut summary_view = HelpView::default();
        summary_view.set_value(&self.text);
    }

    fn contents_size(&self) -> (i32, i32) {
        (600, 500)
    }

    fn validate(&self) -> Result<(), &str> {
        Ok(())
    }
}

impl SummaryActivity {
    pub fn new(
        db_path: Option<&Path>,
        blender_vers: &[BlenderVersion],
        houdini_vers: &[HoudiniVersion],
    ) -> Self {
        // blender text
        let blender_ver_text = if blender_vers.len() > 0 {
            let mut text = format!(
                "\
                <h3>Blender Versions:</h3>\
                \
            "
            );
            for ver in blender_vers.iter() {
                text.push_str(&format!(
                    "\
                <ul>\
                <li>blender [{}.{}.{}]: {:?}\
                </ul>\
                ",
                    ver.version.0, ver.version.1, ver.version.2, &ver.bin_path
                ));
            }
            text
        } else {
            "No blender versions".to_owned()
        };

        // houdini text
        let houini_ver_text = if houdini_vers.len() > 0 {
            let mut text = format!(
                "\
                <h3>Houdini Versions:</h3>\
                \
            "
            );
            for ver in houdini_vers.iter() {
                text.push_str(&format!(
                    "\
                <ul>\
                <li>houdini.py{}_{} [{}.{}.{}]: {:?}\
                </ul>\
                ",
                    ver.python_version.0,
                    ver.python_version.1,
                    ver.version.0,
                    ver.version.1,
                    ver.version.2,
                    &ver.bin_path
                ));
            }
            text
        } else {
            "No Houdini versions".to_owned()
        };

        //

        let text = format!(
            "\
            <h1>Summary</h1>\
            \
            <h3>Database location</h3>\
            {}
            {}
            {}
            \
        ",
            if let Some(path) = db_path {
                path.to_str().unwrap_or("<display error>")
            } else {
                "default location"
            },
            blender_ver_text,
            houini_ver_text,
        );
        SummaryActivity { text }
    }
}
