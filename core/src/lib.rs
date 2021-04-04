#![allow(dead_code)]
#![warn(clippy::all)]

pub use crate::app::*;
pub use crate::config::*;
pub use crate::plugins::plugin::*;
pub use crate::plugins::plugin_manager::*;
pub use crate::resources::data::*;
pub use crate::resources::resource::*;
pub use crate::resources::shared_data::*;
pub use crate::schedule::phase::*;
pub use crate::schedule::scheduler::*;
pub use crate::schedule::system::*;

pub mod app;
pub mod config;

pub mod plugins {
    pub mod plugin;
    pub mod plugin_manager;
}
pub mod resources {
    pub mod data;
    pub mod resource;
    pub mod shared_data;
}
pub mod schedule {
    pub mod phase;
    pub mod scheduler;
    pub mod system;
}
