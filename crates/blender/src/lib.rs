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
        "load",
        py_fn!(
            py,
            load(
                nrg_engine: NRGEngine,
                executable_path: &str,
                file_to_export: &str
            )
        ),
    )?;
    Ok(())
});

fn load(
    py: Python,
    nrg_engine: NRGEngine,
    executable_path: &str,
    file_to_export: &str,
) -> PyResult<bool> {
    let is_running = nrg_engine.is_running(py);
    println!("NRGEngine is running = {}", is_running);

    if !is_running {
        nrg_engine.start(py, executable_path, file_to_export)?;
    }

    Ok(nrg_engine.is_running(py))
}
