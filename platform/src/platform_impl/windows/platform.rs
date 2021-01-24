#![cfg(target_os = "windows")]

mod externs;
mod macros;
mod types;
mod errors;

pub mod library;
pub mod handle;
pub mod watcher;
pub mod window;