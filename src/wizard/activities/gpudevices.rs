use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use super::super::wizard_activity::WizardActivityTrait;
use crate::theme::ITEM_HEIGHT;
use fltk::button::Button;
use fltk::dialog::NativeFileChooser;
use fltk::enums::Align;
use fltk::group::{Flex, Scroll};
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
                    Vec<(Flex, Input, Input)>, // tags
                )>,
            >,
        >,
    >,
    init_data: Vec<(u32, f64, f64, Vec<(String, String)>)>,
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
                TBD\n\
                ",
            );
        layout.end();
        main_layout.fixed(&layout, 128);

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
                        widgets.borrow_mut()[gpu_i].5[i].0.show();
                    }
                    for i in number_of_tags..MAX_GPU_TAGS {
                        widgets.borrow_mut()[gpu_i].5[i].0.hide();
                    }
                    block_layout.layout();
                    main_layout.layout();
                }
            });

            user_inputs.push((block_layout, gpu_name, mem_size, opencl_ver, cuda_ver, gpu_tags));
        }
        main_layout.end();

        // init
        //TBD
        self.widgets.as_ref().unwrap().borrow_mut().append(&mut user_inputs);

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
        })
    }

    fn contents_size(&self) -> (i32, i32) {
        (800, 1024)
    }

    fn validate(&self) -> Result<(), &str> {
        panic!("TBD");
    }
}

impl GpuDevicesActivity {
    pub fn new(init_data: &Vec<(u32, f64, f64, Vec<(String, String)>)>) -> Self {
        GpuDevicesActivity {
            widgets: None,
            init_data: init_data.clone(),
        }
    }

    pub fn get_gpu_devices(&self) -> Vec<(String, u32, f64, f64, Vec<(String, String)>)> {
        if let Some(widgets) = &self.widgets {
            let mut ret = Vec::new();
            for (layout, name_input, mem_input, ocl_input, cuda_input, tags_inputs) in widgets.borrow().iter() {
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
                    ocl_input.value().parse().unwrap(),  // no error check as input is not supposed to be able to return invalid floats
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
