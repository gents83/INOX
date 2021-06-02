#![allow(dead_code)]
#![warn(clippy::all)]

pub use crate::config_compiler::*;
pub use crate::data_watcher::*;
pub use crate::font_compiler::*;
pub use crate::image_compiler::*;
pub use crate::shader_compiler::*;

pub mod config_compiler;
pub mod data_watcher;
pub mod font_compiler;
pub mod image_compiler;
pub mod shader_compiler;
