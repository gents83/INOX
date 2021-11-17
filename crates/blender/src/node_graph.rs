#![allow(dead_code)]

use std::any::type_name;

use pyo3::prelude::*;
use sabi_serialize::{generate_uid_from_string, serialize, Deserialize, Serialize, Uid};

pub type NodeId = Uid;

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

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
pub struct RustNode {
    int_value: u32,
    float_value: f32,
    string_value: String,
    bool_value: bool,
}

impl Node for RustNode {
    fn base_type() -> &'static str {
        "LogicNodeBase"
    }
    fn description() -> &'static str {
        "Node created in Rust"
    }
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
