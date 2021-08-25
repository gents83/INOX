#![allow(dead_code)]

pub use crate::resources::*;
pub use crate::systems::*;
pub use crate::ui_events::*;
pub use crate::ui_properties::*;
pub use egui::*;
use nrg_resources::SharedDataRw;

pub mod resources;
pub mod systems;
pub mod ui_events;
pub mod ui_properties;

pub fn register_resource_types(shared_data: &SharedDataRw) {
    let mut shared_data = shared_data.write().unwrap();
    shared_data.register_type::<UIWidget>();
}

pub fn unregister_resource_types(shared_data: &SharedDataRw) {
    let mut shared_data = shared_data.write().unwrap();
    shared_data.unregister_type::<UIWidget>();
}
