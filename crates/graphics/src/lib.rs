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
    pub data: [f32; 16], // Packed ray data (origin, direction, throughput, etc.)
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct IntersectionPackedData {
    pub data: [f32; 16], // Packed intersection data (hit point, normal, etc.)
}

pub use self::passes::*;

pub mod passes;
