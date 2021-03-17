use super::device::*;
use crate::common::data_formats::*;

#[derive(Clone)]
pub struct RenderPass {
    inner: crate::api::backend::render_pass::RenderPass,
    device: Device,
}

impl RenderPass {
    pub fn create_default(device: &Device, data: &RenderPassData) -> RenderPass {
        RenderPass {
            inner: crate::api::backend::render_pass::RenderPass::create_default(
                &device.inner,
                data,
            ),
            device: device.clone(),
        }
    }

    pub fn destroy(&mut self) {
        self.inner.destroy(&self.device.inner);
    }

    pub fn get_pass(&self) -> &crate::api::backend::render_pass::RenderPass {
        &self.inner
    }

    pub fn begin(&self) {
        self.inner.begin(&self.device.inner);
    }

    pub fn end(&self) {
        self.inner.end(&self.device.inner);
    }
}
