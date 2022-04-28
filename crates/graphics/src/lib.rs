#![warn(clippy::all)]

pub use image::DynamicImage;

pub use crate::common::*;
pub use crate::data::*;
pub use crate::events::*;
pub use crate::fonts::*;
pub use crate::resources::*;
pub use crate::systems::*;

pub mod common;
pub mod data;
pub mod events;
pub mod fonts;
pub mod platform;
pub mod resources;
pub mod systems;
