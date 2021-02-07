#![cfg(target_os = "windows")]

mod macros;
mod types;
mod errors;

pub mod externs;
pub mod handle;
pub mod library;
pub mod utils;
pub mod watcher;
pub mod window;