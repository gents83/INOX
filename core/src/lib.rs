#![allow(dead_code)]
#![warn(clippy::all)]

pub use crate::app::*;
pub use crate::plugins::plugin::*;
pub use crate::plugins::plugin_manager::*;
pub use crate::schedule::phase::*;
pub use crate::schedule::scheduler::*;
pub use crate::schedule::system::*;
pub use crate::schedule::worker::*;

pub mod app;

pub mod plugins {
    pub mod plugin;
    pub mod plugin_manager;
}
pub mod schedule {
    pub mod phase;
    pub mod scheduler;
    pub mod system;
    pub mod worker;
}
