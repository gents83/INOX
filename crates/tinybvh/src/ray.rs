use core::f32;

use crate::ffi;

/// Intersection data.
///
/// Contains intersection distance, as well as barycentric coordinates.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Intersection {
    /// Intersection distance. [`crate::INFINITE`] when empty.
    pub t: f32,
    /// Barycentric weight along the first edge.
    pub u: f32,
    /// Barycentric weight along the second edge.
    pub v: f32,
    /// Primitive index.
    pub prim: u32,
}

impl Intersection {
    /// Create a new intersection.
    ///
    /// The intersection distance defaults to[`crate::INFINITE`] with empty
    /// barycentric coordinates, and primitive.
    pub fn new() -> Self {
        Self {
            t: crate::INFINITE,
            ..Default::default()
        }
    }
}

/// Ray data.
///
/// Origin, distance, and [`Intersection`].
///
/// # Notes
///
/// Padding is unused and required for optimal alignment and performance.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Ray {
    /// Ray origin
    pub origin: [f32; 3],
    pub padding_0: u32,
    /// Ray direction
    pub dir: [f32; 3],
    pub padding_1: u32,
    /// Ray inverse direction.
    /// Automatically computed when using [`Ray::new`].
    pub r_d: [f32; 3],
    pub padding_2: u32,
    /// Ray intersection data.
    pub hit: Intersection,
}

impl Ray {
    /// Create a new ray.
    ///
    /// Automatically computes [`Ray::r_d`].
    pub fn new(origin: [f32; 3], dir: [f32; 3]) -> Self {
        ffi::ray_new(&origin, &dir)
    }
}
