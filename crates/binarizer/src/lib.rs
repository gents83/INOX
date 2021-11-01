#![allow(dead_code)]
#![warn(clippy::all)]

pub use crate::copy_compiler::*;
pub use crate::data_watcher::*;
pub use crate::font_compiler::*;
pub use crate::gltf_compiler::*;
pub use crate::image_compiler::*;
pub use crate::shader_compiler::*;
pub use crate::utils::*;

pub mod data_watcher;

mod copy_compiler;
mod font_compiler;
mod gltf_compiler;
mod image_compiler;
mod shader_compiler;

mod utils;
