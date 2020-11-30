#![cfg(target_os = "windows")]

extern crate nrg_platform;
extern crate vulkan_bindings;

pub use types::*;
pub use utils::*;
pub use device::*;
pub use instance::*;

pub mod device;
pub mod instance;
mod physical_device;
mod shader;

mod macros;
mod types;
mod utils;

mod test;