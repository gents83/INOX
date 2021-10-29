#![warn(clippy::all)]

pub use image::DynamicImage;

pub use crate::common::*;
pub use crate::data::*;
pub use crate::fonts::*;
pub use crate::resources::*;
pub use crate::systems::*;

pub mod api {
    #[cfg(target_os = "ios")]
    #[path = "metal/backend.rs"]
    pub mod backend;

    //Vulkan is supported by Windows, Android, MacOs, Unix
    #[cfg(not(target_os = "ios"))]
    #[path = "vulkan/backend.rs"]
    pub mod backend;
}

pub mod common;
pub mod data;
pub mod fonts;
pub mod resources;
pub mod systems;
mod voxels;
