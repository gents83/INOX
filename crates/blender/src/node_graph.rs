#![allow(dead_code)]

use std::any::type_name;

use nrg_serialize::{generate_uid_from_string, Uid};
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};

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

#[derive(Default)]
pub struct RustNode {
    rust_value: u32,
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
    N: Node + 'static,
{
    println!("Registering node {}", N::node_type());

    let node_name = N::node_type();
    let base_class = N::base_type();
    let description = N::description();
    let value = 10u32;

    let fields: &PyList = PyList::empty(py);
    let field_data: &PyDict = PyDict::from_sequence(
        py,
        (("name", "rust_value"), ("type", "u32"), ("default", value)).into_py(py),
    )?;
    fields.append(field_data)?;

    py.import("NRG")?.getattr("node_tree")?.call_method1(
        "create_node_from_data",
        (node_name, base_class, description, fields),
    )?;

    Ok(true)
}
