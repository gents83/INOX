// Re-export data structures that were previously in compute_ray_generation
// These are simple packed data structures for GPU buffers

use inox_uid::{generate_static_uid_from_string, Uid};

// Buffer IDs for different ray types to ensure separate allocations
pub const BOUNCE_RAYS_ID: Uid = generate_static_uid_from_string("BOUNCE_RAYS");
pub const BOUNCE_RAYS_NEXT_ID: Uid = generate_static_uid_from_string("BOUNCE_RAYS_NEXT");
pub const BOUNCE_INTERSECTIONS_ID: Uid = generate_static_uid_from_string("BOUNCE_INTERSECTIONS");
pub const SHADOW_RAYS_ID: Uid = generate_static_uid_from_string("SHADOW_RAYS");
pub const SHADOW_INTERSECTIONS_ID: Uid = generate_static_uid_from_string("SHADOW_INTERSECTIONS");
pub const AO_RAYS_ID: Uid = generate_static_uid_from_string("AO_RAYS");
pub const AO_INTERSECTIONS_ID: Uid = generate_static_uid_from_string("AO_INTERSECTIONS");

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct RayPackedData {
    pub origin: [f32; 3],
    pub t_min: f32,
    pub direction: [f32; 3],
    pub t_max: f32,
    pub throughput: [f32; 3],
    pub pixel_index: u32,
    pub ray_type: u32,
    pub bounce_count: u32,
    pub flags: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct IntersectionPackedData {
    pub t: f32,
    pub u: f32,
    pub v: f32,
    pub instance_id: i32,
    pub primitive_index: i32,
    pub padding: u32,
    pub _pad: [u32; 2], // Pad to 32 bytes
}

pub use self::passes::*;

pub mod passes;
