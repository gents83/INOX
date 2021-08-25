#![allow(dead_code)]

pub use gizmo::*;
use nrg_resources::SharedDataRw;

pub mod gizmo;

pub fn register_resource_types(shared_data: &SharedDataRw) {
    let mut shared_data = shared_data.write().unwrap();
    shared_data.register_type::<Gizmo>();
}

pub fn unregister_resource_types(shared_data: &SharedDataRw) {
    let mut shared_data = shared_data.write().unwrap();
    shared_data.unregister_type::<Gizmo>();
}
