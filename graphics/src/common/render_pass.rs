use super::device::*;
use crate::{api::backend::Texture, common::data_formats::*, RenderPassId};

#[derive(Clone)]
pub struct RenderPass {
    inner: crate::api::backend::render_pass::RenderPass,
    device: Device,
    id: RenderPassId,
}

impl RenderPass {
    pub fn create_default(device: &Device, id: RenderPassId, data: &RenderPassData) -> RenderPass {
        RenderPass {
            inner: crate::api::backend::render_pass::RenderPass::create_default(
                &device.inner,
                data,
            ),
            device: device.clone(),
            id,
        }
    }
    pub fn create_with_render_target(
        device: &Device,
        id: RenderPassId,
        data: &RenderPassData,
        color: Option<&Texture>,
        depth: Option<&Texture>,
    ) -> RenderPass {
        RenderPass {
            inner: crate::api::backend::render_pass::RenderPass::create_with_render_target(
                &device.inner,
                data,
                color,
                depth,
            ),
            device: device.clone(),
            id,
        }
    }

    pub fn id(&self) -> RenderPassId {
        self.id
    }

    pub fn destroy(&mut self) {
        self.inner.destroy(&self.device.inner);
    }

    pub fn get_pass(&self) -> &crate::api::backend::render_pass::RenderPass {
        &self.inner
    }

    pub fn get_framebuffer_width(&self) -> u32 {
        self.inner.get_framebuffer_width()
    }

    pub fn get_framebuffer_height(&self) -> u32 {
        self.inner.get_framebuffer_height()
    }

    pub fn begin(&self) {
        self.inner.begin(&self.device.inner);
    }

    pub fn end(&self) {
        self.inner.end(&self.device.inner);
    }
}
