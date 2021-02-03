#![warn(clippy::all)]

pub use crate::app::*;
pub use crate::schedule::phase::*;
pub use crate::plugins::plugin::*;
pub use crate::resources::shared_data::*;
pub use crate::schedule::system::*;
pub use crate::plugins::plugin_manager::*;
pub use crate::schedule::scheduler::*;

pub mod app;

pub mod plugins {
    pub mod plugin;
    pub mod plugin_manager;
}
pub mod resources {
    pub mod shared_data;
}
pub mod schedule {
    pub mod phase;
    pub mod scheduler;
    pub mod system;
}
