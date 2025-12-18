// Re-export data structures that were previously in compute_ray_generation
// These are simple packed data structures for GPU buffers

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
