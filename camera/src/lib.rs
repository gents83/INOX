#![allow(dead_code)]
#![warn(clippy::all)]

use nrg_resources::SharedDataRc;

pub use crate::camera::*;
pub use crate::camera_data::*;

pub mod camera;
pub mod camera_data;

pub fn register_resource_types(shared_data: &SharedDataRc) {
    shared_data.register_type_serializable::<Camera>();
}

pub fn unregister_resource_types(shared_data: &SharedDataRc) {
    shared_data.unregister_type_serializable::<Camera>();
}
