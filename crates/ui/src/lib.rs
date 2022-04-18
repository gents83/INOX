pub use crate::color::*;
pub use crate::resources::*;
pub use crate::systems::*;
pub use crate::ui_events::*;
pub use crate::ui_properties::*;
pub use egui::*;
use inox_messenger::MessageHubRc;
use inox_resources::SharedDataRc;

pub mod color;
pub mod resources;
pub mod systems;
pub mod ui_events;
pub mod ui_properties;

pub fn register_resource_types(shared_data: &SharedDataRc, message_hub: &MessageHubRc) {
    shared_data.register_type::<UIWidget>(message_hub);
}

pub fn unregister_resource_types(shared_data: &SharedDataRc, message_hub: &MessageHubRc) {
    shared_data.unregister_type::<UIWidget>(message_hub);
}
