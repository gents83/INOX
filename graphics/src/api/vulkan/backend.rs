#![cfg(target_os = "windows")]

extern crate vulkan_bindings;
pub use vulkan_bindings::*;

pub use types::*;

mod macros;
mod types;

mod test;