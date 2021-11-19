#![crate_type = "cdylib"]
#![allow(clippy::all)]

use pyo3::prelude::*;

mod engine;
mod exporter;
mod macros;
mod node_graph;

use engine::SABIEngine;
use node_graph::ScriptExecution;

// add bindings to the generated python module
// N.B: names: "sabi_blender" must be the name of the `.so` or `.pyd` file
#[pymodule]
#[pyo3(name = "sabi_blender")]
fn sabi_blender(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<SABIEngine>()?;
    m.add_class::<ScriptExecution>()?;

    #[pyfn(m)]
    fn start(sabi_engine: &mut SABIEngine, executable_path: &str) -> PyResult<bool> {
        let is_running = sabi_engine.is_running();

        let result = if !is_running {
            sabi_engine.start(executable_path)?
        } else {
            true
        };
        println!("SABIEngine is running = {}", result);

        Ok(result)
    }

    #[pyfn(m)]
    fn export(
        py: Python,
        sabi_engine: &mut SABIEngine,
        file_to_export: &str,
        load_immediately: bool,
    ) -> PyResult<bool> {
        sabi_engine.export(py, file_to_export, load_immediately)
    }

    #[pyfn(m)]
    fn register_nodes(py: Python, sabi_engine: &SABIEngine) -> PyResult<bool> {
        sabi_engine.register_nodes(py)
    }
    Ok(())
}
