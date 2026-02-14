use inox_uid::{generate_static_uid_from_string, Uid};
use inox_render::{AsBinding, RenderContext, BufferRef};

pub const RAYS_UID: Uid = generate_static_uid_from_string("RAYS_UID");
pub const HITS_UID: Uid = generate_static_uid_from_string("HITS_UID");
pub const SURFACE_UID: Uid = generate_static_uid_from_string("SURFACE_UID");
pub const RADIANCE_UID: Uid = generate_static_uid_from_string("RADIANCE_UID");
pub const THROUGHPUT_UID: Uid = generate_static_uid_from_string("THROUGHPUT_UID");
pub const SHADOW_DATA_UID: Uid = generate_static_uid_from_string("SHADOW_DATA_UID");
pub const NEXT_RAYS_UID: Uid = generate_static_uid_from_string("NEXT_RAYS_UID");

pub const SIZE_OF_DATA_BUFFER_ELEMENT: usize = 4;

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct RayPackedData(pub f32);
impl AsBinding for RayPackedData {
    fn count(&self) -> usize {
        1
    }
    fn size(&self) -> u64 {
        std::mem::size_of::<f32>() as u64
    }
    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut BufferRef) {
        buffer.add_to_gpu_buffer(render_context, &[self.0]);
    }
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct HitPackedData(pub f32);
impl AsBinding for HitPackedData {
    fn count(&self) -> usize {
        1
    }
    fn size(&self) -> u64 {
        std::mem::size_of::<f32>() as u64
    }
    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut BufferRef) {
        buffer.add_to_gpu_buffer(render_context, &[self.0]);
    }
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct SurfacePackedData(pub f32);
impl AsBinding for SurfacePackedData {
    fn count(&self) -> usize {
        1
    }
    fn size(&self) -> u64 {
        std::mem::size_of::<f32>() as u64
    }
    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut BufferRef) {
        buffer.add_to_gpu_buffer(render_context, &[self.0]);
    }
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct RadiancePackedData(pub f32);
impl AsBinding for RadiancePackedData {
    fn count(&self) -> usize {
        1
    }
    fn size(&self) -> u64 {
        std::mem::size_of::<f32>() as u64
    }
    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut BufferRef) {
        buffer.add_to_gpu_buffer(render_context, &[self.0]);
    }
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct ThroughputPackedData(pub f32);
impl AsBinding for ThroughputPackedData {
    fn count(&self) -> usize {
        1
    }
    fn size(&self) -> u64 {
        std::mem::size_of::<f32>() as u64
    }
    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut BufferRef) {
        buffer.add_to_gpu_buffer(render_context, &[self.0]);
    }
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct ShadowDataPackedData(pub f32);
impl AsBinding for ShadowDataPackedData {
    fn count(&self) -> usize {
        1
    }
    fn size(&self) -> u64 {
        std::mem::size_of::<f32>() as u64
    }
    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut BufferRef) {
        buffer.add_to_gpu_buffer(render_context, &[self.0]);
    }
}
