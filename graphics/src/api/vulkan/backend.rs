#![cfg(target_os = "windows")]

extern crate nrg_platform;
extern crate vulkan_bindings;

pub use types::*;
pub use utils::*;
pub use device::*;
pub use instance::*;
pub use material::*;
pub use mesh::*;
pub use texture::*;
pub use render_pass::*;

pub mod device;
pub mod instance;
pub mod material;
pub mod mesh;
pub mod render_pass;
mod physical_device;
mod shader;
mod texture;
mod data_formats;

mod macros;
mod types;
mod utils;

mod test;