#![crate_type = "cdylib"]
#![allow(clippy::all)]

use pyo3::prelude::*;

mod engine;
mod exporter;
mod node_graph;

use engine::NRGEngine;

// add bindings to the generated python module
// N.B: names: "nrg_blender" must be the name of the `.so` or `.pyd` file
#[pymodule]
#[pyo3(name = "nrg_blender")]
fn nrg_blender(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<NRGEngine>()?;

    #[pyfn(m)]
    fn start(nrg_engine: &mut NRGEngine, executable_path: &str) -> PyResult<bool> {
        let is_running = nrg_engine.is_running();

        let result = if !is_running {
            nrg_engine.start(executable_path)?
        } else {
            true
        };
        println!("NRGEngine is running = {}", result);

        Ok(result)
    }

    #[pyfn(m)]
    fn export(
        py: Python,
        nrg_engine: &mut NRGEngine,
        file_to_export: &str,
        load_immediately: bool,
    ) -> PyResult<bool> {
        nrg_engine.export(py, file_to_export, load_immediately)
    }

    #[pyfn(m)]
    fn register_nodes(py: Python, nrg_engine: &NRGEngine) -> PyResult<bool> {
        nrg_engine.register_nodes(py)
    }
    Ok(())
}
