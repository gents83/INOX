#![allow(dead_code)]
#![warn(clippy::all)]

use inox_messenger::MessageHubRc;
use inox_resources::SharedDataRc;

pub use crate::data::*;

pub use crate::camera::*;
pub use crate::hitbox::*;
pub use crate::object::*;
pub use crate::scene::*;
pub use crate::script::*;

pub mod camera;
pub mod data;
pub mod hitbox;
pub mod object;
pub mod scene;
pub mod script;

pub fn register_resource_types(shared_data: &SharedDataRc, message_hub: &MessageHubRc) {
    shared_data.register_type_serializable::<Object>(message_hub);
    shared_data.register_type::<Hitbox>();
    shared_data.register_type_serializable::<Camera>(message_hub);
    shared_data.register_type_serializable::<Script>(message_hub);
    shared_data.register_type_serializable::<Scene>(message_hub);
}

pub fn unregister_resource_types(shared_data: &SharedDataRc) {
    shared_data.unregister_type_serializable::<Object>();
    shared_data.unregister_type::<Hitbox>();
    shared_data.unregister_type_serializable::<Camera>();
    shared_data.unregister_type_serializable::<Script>();
    shared_data.unregister_type_serializable::<Scene>();
}