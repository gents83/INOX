#![allow(dead_code)]
#![warn(clippy::all)]

use sabi_resources::SharedDataRc;

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

pub fn register_resource_types(shared_data: &SharedDataRc) {
    shared_data.register_serializable_type::<CameraData>();
    shared_data.register_serializable_type::<ObjectData>();
    shared_data.register_serializable_type::<SceneData>();

    shared_data.register_serializable_resource_type::<Object>();
    shared_data.register_resource_type::<Hitbox>();
    shared_data.register_serializable_resource_type::<Camera>();
    shared_data.register_serializable_resource_type::<Script>();
    shared_data.register_serializable_resource_type::<Scene>();
}

pub fn unregister_resource_types(shared_data: &SharedDataRc) {
    shared_data.unregister_serializable_resource_type::<Object>();
    shared_data.unregister_resource_type::<Hitbox>();
    shared_data.unregister_serializable_resource_type::<Camera>();
    shared_data.unregister_serializable_resource_type::<Script>();
    shared_data.unregister_serializable_resource_type::<Scene>();

    shared_data.unregister_serializable_type::<CameraData>();
    shared_data.unregister_serializable_type::<ObjectData>();
    shared_data.unregister_serializable_type::<SceneData>();
}
