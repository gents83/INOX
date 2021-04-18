#![warn(clippy::all)]

pub use self::triangle::*;
pub use glam::*;

//Modules

pub mod triangle;

pub type Matrix4 = glam::Mat4;
pub type Matrix3 = glam::Mat3;

pub type Vector2 = glam::Vec2;
pub type Vector3 = glam::Vec3;
pub type Vector4 = glam::Vec4;

#[inline]
pub fn lerp_v2(t: f32, p0: Vector2, p1: Vector2) -> Vector2 {
    Vector2::new(p0.x + t * (p1.x - p0.x), p0.y + t * (p1.y - p0.y))
}
