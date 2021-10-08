#![warn(clippy::all)]

pub use cgmath::*;

pub use crate::angle::*;
pub use crate::matrix::*;
pub use crate::parser::*;
pub use crate::quaternion::*;
pub use crate::random::*;
pub use crate::ray::*;
pub use crate::triangle::*;
pub use crate::vector::*;

pub mod angle;
pub mod matrix;
pub mod parser;
pub mod quaternion;
pub mod random;
pub mod ray;
pub mod triangle;
pub mod vector;

pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Vector4 = cgmath::Vector4<f32>;
pub type Vector2u = cgmath::Vector2<u32>;
pub type Vector3u = cgmath::Vector3<u32>;
pub type Vector4u = cgmath::Vector4<u32>;

pub type Matrix3 = cgmath::Matrix3<f32>;
pub type Matrix4 = cgmath::Matrix4<f32>;

pub type Quaternion = cgmath::Quaternion<f32>;
