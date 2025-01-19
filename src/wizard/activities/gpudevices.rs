use std::cell::RefCell;
use std::rc::Rc;

use super::super::wizard_activity::WizardActivityTrait;
use crate::theme::ITEM_HEIGHT;
use fltk::enums::Align;
use fltk::group::Flex;
use fltk::image::PngImage;
use fltk::input::{FloatInput, Input, IntInput};
use fltk::misc::Spinner;
use fltk::{frame::Frame, prelude::*};

static ICON_DATA: &'static [u8] = include_bytes!("images/browse_noBG.png");

pub struct GpuDevicesActivity {
    widgets: Option<
        Rc<
            RefCell<
                Vec<(
                    Flex,
                    Input,                     // device name
                    IntInput,                  // mem GB
                    FloatInput,                // OpenCL version
                    FloatInput,                // CUDA CC
                    Spinner,                   // tag count spinner
                    Vec<(Flex, Input, Input)>, // tags
                )>,
            >,
        >,
    >,
    init_data: Vec<(String, u32, f64, f64, Vec<(String, String)>)>,
}

impl WizardActivityTrait for GpuDevicesActivity {
    fn start_activity(&mut self) {
        let mut main_layout = Flex::default().column();
        let mut layout = Flex::default().row();
        let mut icon = Frame::default();
        icon.set_image(Some(PngImage::from_data(ICON_DATA).unwrap()));
        layout.fixed(&icon, 144);
        Frame::default()
            .with_align(Align::Inside | Align::Left)
            .with_label(
                "\
                Set up GPU devices for this machine.\n\
                It is really hard to autodetect GPU parameters for different platforms, vendors and operating systems,\n\
                therefore I'd really appreciate if you can fill this information in yourself.\n\
                \n\
                The Name can be anything - name is for you to recognize the device later\n\
                Supported OpenCL version and CUDA Compute Capability - those should be specified in your GPU's\n\
                technical specification list, you can either google it, or use a tool (such as GPUZ)
                "
            );
        layout.end();
        main_layout.fixed(&layout, 128);
        main_layout.fixed(&Frame::default()
            .with_align(Align::Inside | Align::Left)
            .with_label("\
You will be able to use these resources to filter GPUs you want to calculate on

Now for the tags - tags are very special
Tags - is something different DCC nodes may use to distinguish one GPU from another.
The problem is - Redshift, Houdini, Karma... - they all use DIFFERENT ways of enumerating GPUs, which makes it very inconvenient 
to target them.
Tags exist to solve this problem.
For example, here are some known tags currently used by Lifeblood nodes:

houdini_ocl - this tag must consist of device type, vendor name and number,
    as recognized by houdini's env variables HOUDINI_OCL_DEVICETYPE, HOUDINI_OCL_VENDOR and HOUDINI_OCL_DEVICENUMBER
    respectivelly. An example value would be GPU:Intel(R) Corporation:0
    Or some parts may be omitted if there is no ambiguity. like if you have just one intel card - you can just specify 
    GPU:Intel(R) Corporation:
    Or if you have only 2 nvidia cards, you can specify GPU::0 for first one and GPU::1 for the second
    BUT it is up to you to determine which one is actually 0, and which is 1 as seen by houdi

karma_dev - this tag must have form of <card number>/<number of cards>
    For example, if you have 2 Nvidia cards - one of them will have tag value 0/2  and second one - 1/2
    For a single gpu - the value will most probably be just 0/1
    This value represents the Optix device number as seen by Karma"
            )
        , 410);

        const MAX_GPUS_COUNT: usize = 8;
        const MAX_GPU_TAGS: usize = 16;
        const TAG_HEIGHT: i32 = (ITEM_HEIGHT as f64 * 0.75) as i32;

        let mut layout = Flex::default().row();
        let version_count_label = Frame::default().with_label("number of GPUs");
        layout.fixed(&version_count_label, 140);
        let mut version_number_spinner = Spinner::default();
        layout.fixed(&version_number_spinner, 48);
        version_number_spinner.set_step(1.0);
        version_number_spinner.set_minimum(0 as f64);
        version_number_spinner.set_maximum(MAX_GPUS_COUNT as f64);
        version_number_spinner.set_value(0 as f64);
        layout.end();
        main_layout.fixed(&layout, ITEM_HEIGHT);

        let mut user_inputs = Vec::with_capacity(MAX_GPUS_COUNT);
        self.widgets = Some(Rc::new(RefCell::new(Vec::with_capacity(MAX_GPUS_COUNT))));
        for gpu_i in 0..MAX_GPUS_COUNT {
            let mut block_layout = Flex::default().column();

            let mut row_layout = Flex::default().row();
            let label = Frame::default().with_label("gpu name:");
            row_layout.fixed(&label, 128);
            let mut gpu_name = Input::default();
            gpu_name.set_value("");
            //row_layout.end();
            //block_layout.fixed(&row_layout, ITEM_HEIGHT);

            //let mut row_layout = Flex::default().row();
            let label = Frame::default().with_label("memory (GBs):");
            row_layout.fixed(&label, 128);
            let mut mem_size = IntInput::default();
            mem_size.set_value("4");
            row_layout.end();
            block_layout.fixed(&row_layout, ITEM_HEIGHT);

            let mut row_layout = Flex::default().row();
            let label = Frame::default().with_label("OpenCL version:");
            row_layout.fixed(&label, 128);
            let mut opencl_ver = FloatInput::default();
            opencl_ver.set_value("3.0");
            //row_layout.end();
            //block_layout.fixed(&row_layout, ITEM_HEIGHT);

            //let mut row_layout = Flex::default().row();
            let label = Frame::default().with_label("CUDA Compute Compatibility:");
            row_layout.fixed(&label, 200);
            let mut cuda_ver = FloatInput::default();
            cuda_ver.set_value("7.0");
            row_layout.end();
            block_layout.fixed(&row_layout, ITEM_HEIGHT);

            let mut row_layout = Flex::default().row();
            let tags_count_label = Frame::default().with_label("number of tags");
            row_layout.fixed(&tags_count_label, 160);
            let mut tags_number_spinner = Spinner::default();
            row_layout.fixed(&tags_number_spinner, 48);
            tags_number_spinner.set_step(1.0);
            tags_number_spinner.set_minimum(0 as f64);
            tags_number_spinner.set_maximum(MAX_GPU_TAGS as f64);
            tags_number_spinner.set_value(0 as f64);
            row_layout.end();
            block_layout.fixed(&row_layout, ITEM_HEIGHT);

            let mut gpu_tags = Vec::with_capacity(MAX_GPU_TAGS);
            for _ in 0..MAX_GPU_TAGS {
                let mut row_layout = Flex::default().row();
                let label = Frame::default().with_label("tag name/value");
                row_layout.fixed(&label, 256);
                let mut name = Input::default();
                name.set_value("");
                let mut value = Input::default();
                value.set_value("");
                row_layout.end();
                block_layout.fixed(&row_layout, TAG_HEIGHT);
                row_layout.hide();

                gpu_tags.push((row_layout, name, value));
            }
            block_layout.end();
            block_layout.hide();

            // callbacks

            tags_number_spinner.set_callback({
                let widgets = self.widgets.as_ref().unwrap().clone();
                let main_layout = main_layout.clone();
                let block_layout = block_layout.clone();
                move |w| {
                    let number_of_tags = w.value() as usize;
                    for i in 0..number_of_tags {
                        widgets.borrow_mut()[gpu_i].6[i].0.show();
                    }
                    for i in number_of_tags..MAX_GPU_TAGS {
                        widgets.borrow_mut()[gpu_i].6[i].0.hide();
                    }
                    block_layout.layout();
                    main_layout.layout();
                }
            });

            user_inputs.push((
                block_layout,
                gpu_name,
                mem_size,
                opencl_ver,
                cuda_ver,
                tags_number_spinner,
                gpu_tags,
            ));
        }
        main_layout.end();

        // init
        version_number_spinner.set_value(self.init_data.len().min(MAX_GPUS_COUNT) as f64);
        for (init_data, user_input) in self.init_data.iter().zip(user_inputs.iter_mut()) {
            user_input.0.show();
            user_input.1.set_value(&init_data.0);
            user_input.2.set_value(&format!("{}", init_data.1));
            user_input.3.set_value(&format!("{}", init_data.2));
            user_input.4.set_value(&format!("{}", init_data.3));

            user_input
                .5
                .set_value(init_data.4.len().min(MAX_GPU_TAGS) as f64);
            for ((tag, val), tag_widgets) in init_data.4.iter().zip(user_input.6.iter_mut()) {
                tag_widgets.0.show();
                tag_widgets.1.set_value(tag);
                tag_widgets.2.set_value(val);
            }
        }
        self.widgets
            .as_ref()
            .unwrap()
            .borrow_mut()
            .append(&mut user_inputs);

        main_layout.layout();

        // callbacks
        version_number_spinner.set_callback({
            let widgets = self.widgets.as_ref().unwrap().clone();
            move |w| {
                let number_of_versions = w.value() as usize;
                for i in 0..number_of_versions {
                    widgets.borrow_mut()[i].0.show();
                }
                for i in number_of_versions..MAX_GPUS_COUNT {
                    widgets.borrow_mut()[i].0.hide();
                }
                main_layout.layout();
            }
        });
    }

    fn contents_size(&self) -> (i32, i32) {
        (900, 1024)
    }

    fn validate(&self) -> Result<(), &str> {
        if let Some(widgets) = &self.widgets {
            for (layout, name_input, mem_input, ocl_input, cuda_input, _, tags_inputs) in widgets.borrow().iter() {
                if !layout.visible() {
                    break;
                }
                if name_input.value().trim().len() == 0 {
                    return Err("gpu name cannot be empty");
                }
                match i64::from_str_radix(&mem_input.value(), 10) {
                    Err(_) => return Err("failed to parse memory"),
                    Ok(x) if x < 0 => return Err("memory cannot be negative"),
                    Ok(_) => (),
                }
                match ocl_input.value().parse::<f64>() {
                    Err(_) => return Err("failed to parse OCL version"),
                    Ok(x) if x < 0_f64 => return Err("OCL version cannot be negative"),
                    Ok(_) => (),
                }
                match cuda_input.value().parse::<f64>() {
                    Err(_) => return Err("failed to parse CUDA CC"),
                    Ok(x) if x < 0_f64 => return Err("CUDA CC cannot be negative"),
                    Ok(_) => (),
                }
                for (tag_layout, tag_name, _)  in tags_inputs.iter() {
                    if !tag_layout.visible() {
                        break;
                    }
                    if tag_name.value().trim().len() == 0 {
                        return Err("tag name cannot be empty");
                    }
                    // tag value can be empty, i guess, why not...
                }
            }
        }

        Ok(())
    }
}

impl GpuDevicesActivity {
    pub fn new(init_data: &Vec<(String, u32, f64, f64, Vec<(String, String)>)>) -> Self {
        GpuDevicesActivity {
            widgets: None,
            init_data: init_data.clone(),
        }
    }

    pub fn get_gpu_devices(&self) -> Vec<(String, u32, f64, f64, Vec<(String, String)>)> {
        if let Some(widgets) = &self.widgets {
            let mut ret = Vec::new();
            for (layout, name_input, mem_input, ocl_input, cuda_input, _, tags_inputs) in
                widgets.borrow().iter()
            {
                if !layout.visible() {
                    break;
                }
                let mut tags = Vec::new();
                for (tag_layout, tag_name_input, tag_value_input) in tags_inputs.iter() {
                    if !tag_layout.visible() {
                        break;
                    }
                    tags.push((tag_name_input.value(), tag_value_input.value()));
                }

                ret.push((
                    name_input.value(),
                    u32::from_str_radix(&mem_input.value(), 10).unwrap(),
                    ocl_input.value().parse().unwrap(), // no error check as input is not supposed to be able to return invalid floats
                    cuda_input.value().parse().unwrap(),
                    tags,
                ));
            }
            ret
        } else {
            Vec::new()
        }
    }
}
