#![allow(dead_code)]

use std::any::type_name;

use pyo3::prelude::*;
use sabi_serialize::{generate_uid_from_string, serialize, Deserialize, Serialize, Uid};

use crate::{implement_node, implement_output_pin};

pub type NodeId = Uid;

implement_output_pin!(ScriptExecution);

pub trait Node: Default + Send + Sync + 'static {
    #[inline]
    fn python<'a>() -> Python<'a> {
        unsafe { Python::assume_gil_acquired() }
    }
    fn type_id() -> NodeId {
        generate_uid_from_string(Self::node_type())
    }
    fn node_type() -> &'static str {
        type_name::<Self>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
    }
    fn base_type() -> &'static str;
    fn description() -> &'static str;
}

pub fn register_node<N>(py: Python) -> PyResult<bool>
where
    N: Node + 'static + Serialize,
{
    println!("Registering node {}", N::node_type());

    let node_name = N::node_type();
    let base_class = N::base_type();
    let description = N::description();
    let serialized_class = serialize(&N::default());

    py.import("SABI")?.getattr("node_tree")?.call_method1(
        "create_node_from_data",
        (node_name, base_class, description, serialized_class),
    )?;

    Ok(true)
}

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
pub struct RustNode {
    in_int_value: u32,
    in_float_value: f32,
    in_string_value: String,
    in_bool_value: bool,
    out_execute: ScriptExecution,
    out_int_value: u32,
    out_float_value: f32,
    out_string_value: String,
    out_bool_value: bool,
}
implement_node!(RustNode, "Example node created from Rust");

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
pub struct ScriptInitNode {
    out_execute: ScriptExecution,
}
implement_node!(
    ScriptInitNode,
    "Node will be called when starting the script"
);

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
pub struct MoveNode {
    in_run: ScriptExecution,
    in_x: f32,
    in_y: f32,
    in_z: f32,
}
implement_node!(MoveNode, "Node will move object in space");
