#![allow(dead_code)]
#![warn(clippy::all)]

use nrg_resources::SharedDataRc;

pub use crate::data::*;

pub use crate::hitbox::*;
pub use crate::object::*;
pub use crate::scene::*;
pub use crate::transform::*;

pub mod data;
pub mod hitbox;
pub mod object;
pub mod scene;
pub mod transform;

pub fn register_resource_types(shared_data: &SharedDataRc) {
    shared_data.register_type::<Hitbox>();
    shared_data.register_type_serializable::<Object>();
    shared_data.register_type_serializable::<Scene>();
    shared_data.register_type::<Transform>();
}

pub fn unregister_resource_types(shared_data: &SharedDataRc) {
    shared_data.unregister_type::<Hitbox>();
    shared_data.unregister_type_serializable::<Object>();
    shared_data.unregister_type_serializable::<Scene>();
    shared_data.unregister_type::<Transform>();
}
