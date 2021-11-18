#![allow(dead_code)]

use std::any::type_name;

use pyo3::prelude::*;
use sabi_serialize::{generate_uid_from_string, serialize, Deserialize, Serialize, Uid};

pub type NodeId = Uid;

#[macro_export]
macro_rules! implement_output_pin {
    ($Type:ident) => {
        #[pyclass(module = "sabi_blender")]
        #[derive(Serialize, Deserialize)]
        #[serde(crate = "sabi_serialize")]
        pub struct $Type {
            type_name: String,
        }
        impl Default for $Type {
            fn default() -> Self {
                let type_name = std::any::type_name::<Self>()
                    .split(':')
                    .collect::<Vec<&str>>()
                    .last()
                    .unwrap()
                    .to_string();
                Self { type_name }
            }
        }
    };
}

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
