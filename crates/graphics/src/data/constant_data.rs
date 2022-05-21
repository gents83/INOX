use std::mem::size_of;

use crate::{AsBufferBinding, DataBuffer, RenderContext};

pub const CONSTANT_DATA_FLAGS_NONE: u32 = 0;
pub const CONSTANT_DATA_FLAGS_SUPPORT_SRGB: u32 = 1;

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct ConstantData {
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub screen_width: f32,
    pub screen_height: f32,
    pub flags: u32,
}

impl Default for ConstantData {
    fn default() -> Self {
        Self {
            view: [[0.; 4]; 4],
            proj: [[0.; 4]; 4],
            screen_width: 0.,
            screen_height: 0.,
            flags: CONSTANT_DATA_FLAGS_NONE,
        }
    }
}

impl AsBufferBinding for ConstantData {
    fn size(&self) -> u64 {
        size_of::<ConstantData>() as _
    }

    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut DataBuffer) {
        buffer.add_to_gpu_buffer(render_context, &[*self]);
    }
}
