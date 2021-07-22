#![allow(dead_code)]
#![warn(clippy::all)]

pub use crate::data::*;

pub use crate::hitbox::*;
pub use crate::object::*;
pub use crate::scene::*;
pub use crate::transform::*;

pub mod data;
pub mod hitbox;
pub mod object;
pub mod scene;
pub mod transform;
