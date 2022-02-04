#![warn(clippy::all)]
#![allow(dead_code)]

pub use inox_filesystem::*;

pub use self::macros::*;
pub mod macros;

#[cfg(debug_assertions)]
pub use self::profiler::*;

#[cfg(debug_assertions)]
pub mod profiler;

//Using Chrome browser for profiling
//https://www.chromium.org/developers/how-tos/trace-event-profiling-tool
//go to chrome://tracing and click on "Load"
