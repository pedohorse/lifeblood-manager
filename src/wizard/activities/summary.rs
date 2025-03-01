use crate::wizard::wizard_data::RedshiftVersion;

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
        scratch_path: Option<&Path>,
        blender_vers: &[BlenderVersion],
        houdini_vers: &[HoudiniVersion],
        houdini_tools_paths: &[&Path],
        redshift_vers: &[RedshiftVersion],
        gpu_devices: &[(String, u32, f64, f64, Vec<(String, String)>)],
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
            "<h4>No blender versions</h4>".to_owned()
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
            "<h4>No Houdini versions</h4>".to_owned()
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

        // redshift text
        let redshift_ver_text = if redshift_vers.len() > 0 {
            let mut text = format!(
                "\
                <h3>Redshift Versions:</h3>\
                <ul>\
                \
            "
            );
            for ver in redshift_vers.iter() {
                text.push_str(&format!(
                    "<li>redshift [{}.{}.{}]: {:?}",
                    ver.version.0,
                    ver.version.1,
                    ver.version.2,
                    &ver.bin_path
                ));
            }
            text.push_str("</ul>");
            text
        } else {
            "<h4>No Redshift versions</h4>".to_owned()
        };

        // gpu devices
        let gpu_summary = if gpu_devices.len() > 0 {
            let mut text = format!(
                "\
                <h3>GPU devices</h3>\
                <ul>
                ",
            );
            for (dev_name, dev_mem, ocl, cuda, tags) in gpu_devices.iter() {
                text.push_str(&format!("<li>{dev_name} ({dev_mem}GB): ocl:{ocl}, cuda:{cuda}"));
                if tags.len() > 0 {
                    text.push_str("<ul>");
                    for (tag_name, tag_val) in tags.iter() {
                        text.push_str(&format!("<li>{tag_name}={tag_val}"));
                    }
                    text.push_str("</ul>");
                }
            }
            text.push_str("</ul>");
            text
        } else {
            format!("<h3>No GPU devices</h3>")
        };

        //

        let config_summary = format!(
            "\
            <h5>Note: existing config files will be overwritten!</h5>\
            \
            <h3>Database location:</h3>\
            {}\n\
            <h3>Shared Scratch location:</h3>\
            {}\n\
            {}\n\
            {}\n\
            {}\n\
            {}\n\
            ",
            if let Some(path) = db_path {
                path.to_str().unwrap_or("<display error>")
            } else {
                "default location"
            },
            if let Some(path) = scratch_path {
                path.to_str().unwrap_or("<display error>")
            } else {
                "default location"
            },
            blender_ver_text,
            houini_ver_text,
            redshift_ver_text,
            gpu_summary,
        );

        // tools summary text
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
