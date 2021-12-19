#![allow(dead_code)]

pub use gizmo::*;
use sabi_resources::SharedDataRc;

pub mod gizmo;

pub fn register_resource_types(shared_data: &SharedDataRc) {
    shared_data.register_resource_type::<Gizmo>();
}

pub fn unregister_resource_types(shared_data: &SharedDataRc) {
    shared_data.unregister_resource_type::<Gizmo>();
}
