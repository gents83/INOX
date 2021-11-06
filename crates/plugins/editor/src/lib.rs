#![warn(clippy::all)]

pub use events::*;

mod config;
pub mod editor;

mod editor_updater;
mod events;
mod resources;
pub mod systems;
mod widgets;
