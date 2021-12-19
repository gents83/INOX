#![allow(dead_code)]

pub use crate::color::*;
pub use crate::resources::*;
pub use crate::systems::*;
pub use crate::ui_events::*;
pub use crate::ui_properties::*;
pub use egui::*;
use sabi_resources::SharedDataRc;
use systems::config::Config;

pub mod color;
pub mod resources;
pub mod systems;
pub mod ui_events;
pub mod ui_properties;

pub fn register_resource_types(shared_data: &SharedDataRc) {
    shared_data.register_serializable_type::<Config>();

    shared_data.register_resource_type::<UIWidget>();
}

pub fn unregister_resource_types(shared_data: &SharedDataRc) {
    shared_data.unregister_resource_type::<UIWidget>();

    shared_data.unregister_serializable_type::<Config>();
}
