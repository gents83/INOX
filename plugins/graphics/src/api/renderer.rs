use std::collections::HashMap;

use nrg_math::*;
use nrg_platform::Handle;

use super::viewport::*;
use super::rasterizer::*;
use super::device::*;
use super::instance::*;
use super::pipeline::*;
use super::render_pass::*;


pub const DEFAULT_RENDER_PASS:&str = "Default";

pub struct Renderer {
    pub instance: Instance,
    pub device: Device,
    viewport: Viewport,
    scissors: Scissors,
    rasterizer: Rasterizer,
    render_passes: HashMap<String, RenderPass>,
    pipelines: HashMap<String, Pipeline>,
}


impl Renderer {
    pub fn new(handle: &Handle, debug_enabled: bool) -> Self {        
        let instance = Instance::create(handle, debug_enabled);
        let device = Device::create(&instance);
        Renderer {
            instance: instance,
            device: device,
            viewport: Viewport::default(),
            scissors: Scissors::default(),
            rasterizer: Rasterizer::default(),
            render_passes: HashMap::new(),
            pipelines: HashMap::new(),
        }
    }

    pub fn get_default_render_pass(&mut self) -> &RenderPass {
        let rp = self.render_passes
            .entry(String::from(DEFAULT_RENDER_PASS))
            .or_insert(RenderPass::create_default(&self.device));
        rp
    }

    pub fn get_render_pass(&self, name:String) -> &RenderPass {
        self.render_passes.get(&name).unwrap()
    }

    pub fn add_pipeline(&mut self, name:String, pipeline: Pipeline) -> &Pipeline {
        self.pipelines.insert(name.clone(), pipeline);
        self.get_pipeline(name)
    }

    pub fn get_pipeline(&self, name:String) -> &Pipeline {
        self.pipelines.get(&name).unwrap()
    }

    pub fn set_viewport_size(&mut self, size:Vector2u) -> &mut Self {
        self.viewport.width = size.x as _;
        self.viewport.height = size.y as _;
        self.scissors.width = self.viewport.width;
        self.scissors.height = self.viewport.height; 
        self
    }

    pub fn begin_frame(&mut self) -> bool{
        self.device.begin_frame()
    }
    
    pub fn end_frame(&mut self) -> bool{
        self.device.end_frame();

        //TEMP
        self.device.submit()
    }

    pub fn process_pipelines(&mut self) {
        for (_name, pipeline) in self.pipelines.iter_mut() {
            pipeline.begin();

            
            pipeline.end();
        }
    }
}


impl Drop for Renderer {
    fn drop(&mut self) {
        self.device.destroy();        
        self.instance.destroy();
    }
}