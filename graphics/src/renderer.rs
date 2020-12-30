use nrg_math::*;
use nrg_platform::Handle;

use crate::viewport::*;
use crate::rasterizer::*;
use crate::device::*;
use crate::instance::*;
use crate::render_pass::*;

pub struct Renderer {
    pub instance: Instance,
    pub device: Device,
    viewport: Viewport,
    scissors: Scissors,
    rasterizer: Rasterizer,
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
        }
    }

    pub fn create_default_render_pass(&self) -> RenderPass {
        RenderPass::create_default(&self.device)
    }

    pub fn set_viewport_size(&mut self, size:Vector2u) -> &mut Self {
        self.viewport.width = size.x as _;
        self.viewport.height = size.y as _;
        self.scissors.width = self.viewport.width;
        self.scissors.height = self.viewport.height; 
        self
    }

    pub fn begin_frame(&mut self) {
        self.device.begin_frame();
    }
    
    pub fn end_frame(&mut self) {
        self.device.end_frame();

        //TEMP
        self.device.submit();
    }

}


impl Drop for Renderer {
    fn drop(&mut self) {
        self.device.destroy();        
        self.instance.destroy();
    }
}