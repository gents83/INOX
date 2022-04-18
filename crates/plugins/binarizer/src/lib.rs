#![cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
#![warn(clippy::all)]

pub use crate::compilers::*;
pub use crate::plugin::*;
pub use crate::system::*;
pub use crate::utils::*;

mod compilers;
pub mod plugin;
mod system;
pub mod utils;
