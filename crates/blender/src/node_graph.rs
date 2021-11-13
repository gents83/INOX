#![allow(dead_code)]

use std::any::type_name;

use nrg_serialize::{generate_uid_from_string, Uid};
use pyo3::{prelude::*, types::PyDict, PyClass};

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

#[pyclass]
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

#[pymethods]
impl RustNode {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }
    #[classattr]
    fn type_id() -> u128 {
        <Self as Node>::type_id().as_u128()
    }
    #[classattr]
    fn node_type() -> &'static str {
        <Self as Node>::node_type()
    }
    #[classattr]
    fn base_type() -> &'static str {
        <Self as Node>::base_type()
    }
    #[classattr]
    fn description() -> &'static str {
        <Self as Node>::description()
    }
    #[classattr]
    fn fields<'a>() -> &'a PyDict {
        let py = Self::python();
        let _fields = PyDict::new(py);
        let py_dict = PyDict::new(py);
        py_dict
    }
}

pub fn register_node<N>(py: Python) -> PyResult<bool>
where
    N: Node + 'static + PyClass,
{
    println!("Registering node {}", N::node_type());

    let module = py.import("NRG.nrg_blender")?;
    module.add_class::<N>()?;
    let class = module.getattr(N::node_type())?;

    py.import("NRG")?
        .getattr("node_tree")?
        .call_method1("create_node_from_data", (class,))?;

    println!(
        "{} in Rust exists? -> {}",
        N::node_type(),
        class.getattr("__name__")?.extract::<String>()?
    );

    Ok(true)
}
