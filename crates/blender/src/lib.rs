#![crate_type = "cdylib"]
#![allow(clippy::all)]

extern crate cpython;

mod engine;
mod exporter;
use crate::engine::*;

use cpython::{py_fn, py_module_initializer, PyResult, Python};

// add bindings to the generated python module
// N.B: names: "nrg_blender" must be the name of the `.so` or `.pyd` file
py_module_initializer!(nrg_blender, initnrg_blender, PyInit_nrg_blender, |py, m| {
    m.add(py, "__doc__", "This module is implemented in Rust")?;
    m.add_class::<NRGEngine>(py)?;
    m.add(
        py,
        "start",
        py_fn!(py, start(nrg_engine: NRGEngine, executable_path: &str,)),
    )?;
    m.add(
        py,
        "export",
        py_fn!(py, export(nrg_engine: NRGEngine, file_to_export: &str,)),
    )?;
    Ok(())
});

fn start(py: Python, nrg_engine: NRGEngine, executable_path: &str) -> PyResult<bool> {
    let is_running = nrg_engine.is_running(py);

    let result = if !is_running {
        nrg_engine.start(py, executable_path)?
    } else {
        true
    };
    println!("NRGEngine is running = {}", result);

    Ok(result)
}

fn export(py: Python, nrg_engine: NRGEngine, file_to_export: &str) -> PyResult<bool> {
    nrg_engine.export(py, file_to_export)
}
