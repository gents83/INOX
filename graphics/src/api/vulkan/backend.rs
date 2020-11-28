#![cfg(target_os = "windows")]

extern crate nrg_platform;
extern crate vulkan_bindings;

pub use vulkan_bindings::*;
pub use nrg_platform::*;

pub use types::*;
pub use utils::*;
pub use device::*;
pub use instance::*;

pub mod device;
pub mod instance;

mod macros;
mod types;
mod utils;

mod test;