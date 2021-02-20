use super::device::*;

#[derive(Clone)]
pub struct RenderPass {
    inner : crate::api::backend::render_pass::RenderPass,
    device: Device,
}


impl RenderPass {
    pub fn create_default(device: &Device) -> RenderPass {
        RenderPass {
            inner: crate::api::backend::render_pass::RenderPass::create_default(&device.inner),
            device: device.clone(),
        }
    }

    pub fn destroy(&mut self) {
        self.inner.destroy(&self.device.inner);
    }

    pub fn get_pass(&self) ->  &crate::api::backend::render_pass::RenderPass {
        &self.inner
    }

    pub fn begin(&self) {
        self.inner.begin(&self.device.inner);
    }

    pub fn end(&self) {
        self.inner.end(&self.device.inner);
    }
}
