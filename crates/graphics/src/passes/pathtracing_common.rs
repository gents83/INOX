use inox_serialize::{Deserialize, Serialize};
use inox_uid::{generate_static_uid_from_string, Uid};

pub const NEXT_RAYS_UID: Uid = generate_static_uid_from_string("NEXT_RAYS_BUFFER");
pub const SURFACE_DATA_UID: Uid = generate_static_uid_from_string("SURFACE_DATA_BUFFER");
pub const GEOMETRY_BUFFER_UID: Uid = generate_static_uid_from_string("GEOMETRY_BUFFER");
pub const SCENE_BUFFER_UID: Uid = generate_static_uid_from_string("SCENE_BUFFER");
pub const DISPATCH_INDIRECT_BUFFER_UID: Uid = generate_static_uid_from_string("DISPATCH_INDIRECT_BUFFER");

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct DispatchIndirectArgs {
    pub workgroup_count_x: u32,
    pub workgroup_count_y: u32,
    pub workgroup_count_z: u32,
}

impl inox_render::AsBinding for DispatchIndirectArgs {
    fn is_dirty(&self) -> bool {
        false
    }
    fn set_dirty(&mut self, _is_dirty: bool) {}
    fn size(&self) -> u64 {
        std::mem::size_of::<DispatchIndirectArgs>() as u64
    }
    fn fill_buffer(&self, render_context: &inox_render::RenderContext, buffer: &mut inox_render::BufferRef) {
        buffer.add_to_gpu_buffer(render_context, &[self.workgroup_count_x, self.workgroup_count_y, self.workgroup_count_z]);
    }
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(crate = "inox_serialize")]
pub struct Ray {
    pub origin: [f32; 3],
    pub t_min: f32,
    pub direction: [f32; 3],
    pub t_max: f32,
    pub throughput: [f32; 3],
    pub pixel_index: u32,
    pub depth: u32,
    pub padding: [f32; 3],
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(crate = "inox_serialize")]
pub struct RayHit {
    pub instance_id: u32,
    pub primitive_index: u32,
    pub barycentrics: [f32; 2],
    pub t: f32,
    pub pixel_index: u32,
    pub padding: [u32; 2],
    pub direction: [f32; 3],
    pub pad_dir: f32,
    pub throughput: [f32; 3],
    pub pad_thr: f32,
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(crate = "inox_serialize")]
pub struct ShadowRay {
    pub origin: [f32; 3],
    pub t_max: f32,
    pub direction: [f32; 3],
    pub radiance: u32,
    pub contribution: [f32; 3],
    pub pixel_index: u32,
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(crate = "inox_serialize")]
pub struct SurfaceData {
    pub position: [f32; 3],
    pub material_index: i32,
    pub normal: [f32; 3],
    pub flags: u32,
    pub uv: [f32; 2],
    pub roughness: f32,
    pub metallic: f32,
    pub albedo: [f32; 3],
    pub padding: f32,
    pub tangent: [f32; 4],
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(crate = "inox_serialize")]
pub struct PathTracingCounters {
    pub ray_count: u32,
    pub hit_count: u32,
    pub shadow_ray_count: u32,
    pub extension_ray_count: u32,
    pub next_ray_count: u32,
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct RadiancePackedData(pub f32);

impl inox_render::AsBinding for RadiancePackedData {
    fn is_dirty(&self) -> bool {
        false
    }
    fn set_dirty(&mut self, _is_dirty: bool) {}
    fn size(&self) -> u64 {
        std::mem::size_of::<f32>() as u64
    }
    fn fill_buffer(&self, render_context: &inox_render::RenderContext, buffer: &mut inox_render::BufferRef) {
        buffer.add_to_gpu_buffer(render_context, &[self.0]);
    }
}
