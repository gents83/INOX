use crate::device::*;

pub struct RenderPass(super::api::backend::render_pass::RenderPass);


impl RenderPass {
    pub fn create_default(device: &Device) -> RenderPass {
        RenderPass(super::api::backend::render_pass::RenderPass::create_default(&device.inner))
    }

    pub fn destroy(&mut self, device:&Device) {
        self.0.destroy(&device.inner);
    }

    pub fn get_pass(&self) ->  &super::api::backend::render_pass::RenderPass {
        &self.0
    }

    pub fn begin(&self, device:&Device) {
        self.0.begin(&device.inner);
    }

    pub fn end(&self, device:&Device) {
        self.0.end(&device.inner);
    }
}
