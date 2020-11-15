#![cfg(target_os = "windows")]

extern crate nrg_platform;
extern crate vulkan_bindings;

pub use vulkan_bindings::*;
pub use nrg_platform::*;

pub use types::*;
pub use utils::*;

mod macros;
mod types;
mod utils;

mod test;