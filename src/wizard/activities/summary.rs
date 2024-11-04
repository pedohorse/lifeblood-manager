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
        show_config_part: bool,
        show_tools_part: bool,
        db_path: Option<&Path>,
        blender_vers: &[BlenderVersion],
        houdini_vers: &[HoudiniVersion],
        houdini_tools_paths: &[&Path],
    ) -> Self {
        // blender text
        let blender_ver_text = if blender_vers.len() > 0 {
            let mut text = format!(
                "\
                <h3>Blender Versions:</h3>\
                <ul>\
                \
            "
            );
            for ver in blender_vers.iter() {
                text.push_str(&format!(
                    "<li>blender [{}.{}.{}]: {:?}",
                    ver.version.0, ver.version.1, ver.version.2, &ver.bin_path
                ));
            }
            text.push_str("</ul>");
            text
        } else {
            "<br>No blender versions".to_owned()
        };

        // houdini text
        let houini_ver_text = if houdini_vers.len() > 0 {
            let mut text = format!(
                "\
                <h3>Houdini Versions:</h3>\
                <ul>\
                \
            "
            );
            for ver in houdini_vers.iter() {
                text.push_str(&format!(
                    "<li>houdini.py{}_{} [{}.{}.{}]: {:?}",
                    ver.python_version.0,
                    ver.python_version.1,
                    ver.version.0,
                    ver.version.1,
                    ver.version.2,
                    &ver.bin_path
                ));
            }
            text.push_str("</ul>");
            text
        } else {
            "<br>No Houdini versions".to_owned()
        };

        let houdini_tools_text = if houdini_tools_paths.len() > 0 {
            let mut text = "\
                <h4>Submitting tools for Houdini installed to:</h4>\
                <ul>\
                \
            "
            .to_owned();
            for path in houdini_tools_paths.iter() {
                text.push_str(&format!("<li>{}", path.to_string_lossy(),));
            }
            text.push_str("</ul>");
            text
        } else {
            "<br>No Lifeblood submitting tools for Houdini will be installed".to_owned()
        };

        //

        let config_summary = format!(
            "\
            <h5>Note: existing config files will be overwritten!</h5>\
            \
            <h3>Database location</h3>\
            {}\n\
            {}\n\
            {}\n\
            ",
            if let Some(path) = db_path {
                path.to_str().unwrap_or("<display error>")
            } else {
                "default location"
            },
            blender_ver_text,
            houini_ver_text,
        );

        let tools_summary = format!(
            "\
            <h3>Tools:</h3>\
            {}\
            ",
            houdini_tools_text
        );

        let text = format!(
            "\
            <h1>Summary</h1>\
            \
            {}\
            {}\
            \
            ",
            if show_config_part {
                &config_summary
            } else {
                ""
            },
            if show_tools_part { &tools_summary } else { "" },
        );
        SummaryActivity { text }
    }
}
