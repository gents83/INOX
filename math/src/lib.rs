#![warn(clippy::all)]

pub use cgmath::*;

pub use crate::angle::*;
pub use crate::matrix::*;
pub use crate::quaternion::*;
pub use crate::triangle::*;
pub use crate::vector::*;

pub mod angle;
pub mod matrix;
pub mod quaternion;
pub mod triangle;
pub mod vector;

pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Vector4 = cgmath::Vector4<f32>;

pub type Matrix3 = cgmath::Matrix3<f32>;
pub type Matrix4 = cgmath::Matrix4<f32>;

pub type Quaternion = cgmath::Quaternion<f32>;
