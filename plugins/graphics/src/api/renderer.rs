use std::{collections::HashMap, path::PathBuf};

use crate::fonts::font::*;
use nrg_math::*;
use nrg_platform::Handle;

use super::device::*;
use super::instance::*;
use super::pipeline::*;
use super::rasterizer::*;
use super::render_pass::*;
use super::viewport::*;

pub const DEFAULT_RENDER_PASS: &str = "Default";
pub const DEFAULT_FONT_PIPELINE: &str = "Font_Pipeline";
const FONT_VS_PATH: &str = "C:\\PROJECTS\\NRG\\data\\shaders\\compiled\\text_shader_vert.spv";
const FONT_FRAG_PATH: &str = "C:\\PROJECTS\\NRG\\data\\shaders\\compiled\\text_shader_frag.spv";

pub struct Renderer {
    pub instance: Instance,
    pub device: Device,
    viewport: Viewport,
    scissors: Scissors,
    rasterizer: Rasterizer,
    render_passes: HashMap<String, RenderPass>,
    pipelines: HashMap<String, Pipeline>,
    fonts: HashMap<String, Option<Font>>,
}

impl Renderer {
    pub fn new(handle: &Handle, debug_enabled: bool) -> Self {
        let instance = Instance::create(handle, debug_enabled);
        let device = Device::create(&instance);
        Renderer {
            instance,
            device,
            viewport: Viewport::default(),
            scissors: Scissors::default(),
            rasterizer: Rasterizer::default(),
            render_passes: HashMap::new(),
            pipelines: HashMap::new(),
            fonts: HashMap::new(),
        }
    }

    pub fn get_render_pass(&self, name: String) -> &RenderPass {
        self.render_passes.get(&name).unwrap()
    }

    pub fn get_fonts_count(&self) -> usize {
        self.fonts.len()
    }

    pub fn get_font(&mut self, font_path: PathBuf) -> &mut Option<Font> {
        self.fonts
            .get_mut(&font_path.to_str().unwrap().to_string())
            .unwrap()
    }

    pub fn get_default_font(&mut self) -> &mut Option<Font> {
        self.fonts.iter_mut().next().unwrap().1
    }

    pub fn add_pipeline(&mut self, name: String, pipeline: Pipeline) -> &Pipeline {
        self.pipelines.insert(name.clone(), pipeline);
        self.get_pipeline(name)
    }

    pub fn get_pipeline(&self, name: String) -> &Pipeline {
        self.pipelines.get(&name).unwrap()
    }

    pub fn set_viewport_size(&mut self, size: Vector2u) -> &mut Self {
        self.viewport.width = size.x as _;
        self.viewport.height = size.y as _;
        self.scissors.width = self.viewport.width;
        self.scissors.height = self.viewport.height;
        self
    }

    pub fn begin_frame(&mut self) -> bool {
        self.load_data();
        self.device.begin_frame()
    }

    pub fn end_frame(&mut self) -> bool {
        self.device.end_frame();

        //TEMP
        self.device.submit()
    }

    pub fn process_pipelines(&mut self) {
        for (_name, pipeline) in self.pipelines.iter_mut() {
            pipeline.begin();

            pipeline.end();
        }
        for (_name, option) in self.fonts.iter_mut() {
            if let Some(ref mut font) = option {
                font.render()
            }
        }
    }

    pub fn request_font(&mut self, font_path: PathBuf) {
        self.fonts
            .entry(font_path.to_str().unwrap().to_string())
            .or_insert(None);
    }
}

impl Renderer {
    fn load_data(&mut self) {
        let device = &self.device;
        self.fonts.iter_mut().for_each(|(path, option)| {
            if option.is_none() {
                let font_pipeline = {
                    let def_rp = RenderPass::create_default(device);
                    Pipeline::create(device, FONT_VS_PATH, FONT_FRAG_PATH, def_rp)
                };
                option.get_or_insert(Font::new(device, font_pipeline, PathBuf::from(path)));
            }
            if let Some(ref mut font) = option {
                font.create_meshes();
            }
        });
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.device.destroy();
        self.instance.destroy();
    }
}
