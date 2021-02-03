use super::device::*;

pub struct RenderPass {
    inner : super::backend::render_pass::RenderPass,
    device: Device,
}


impl RenderPass {
    pub fn create_default(device: &Device) -> RenderPass {
        let inner_device = device.inner.borrow();
        RenderPass {
            inner: super::backend::render_pass::RenderPass::create_default(&inner_device),
            device: device.clone(),
        }
    }

    pub fn destroy(&mut self) {
        let inner_device = self.device.inner.borrow();
        self.inner.destroy(&inner_device);
    }

    pub fn get_pass(&self) ->  &super::backend::render_pass::RenderPass {
        &self.inner
    }

    pub fn begin(&self) {
        let inner_device = self.device.inner.borrow();
        self.inner.begin(&inner_device);
    }

    pub fn end(&self) {
        let inner_device = self.device.inner.borrow();
        self.inner.end(&inner_device);
    }
}
