use pyo3::prelude::*;

mod engine;
mod exporter;

use engine::SABIEngine;

// add bindings to the generated python module
// N.B: names: "inox_blender" must be the name of the `.so` or `.pyd` file
#[pymodule]
#[pyo3(name = "inox_blender")]
fn inox_blender(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<SABIEngine>()?;

    #[pyfn(m)]
    fn start(inox_engine: &mut SABIEngine) -> PyResult<bool> {
        let is_running = inox_engine.is_running();

        let result = if !is_running {
            inox_engine.start()?
        } else {
            true
        };
        println!("SABIEngine is running = {}", result);

        Ok(result)
    }

    #[pyfn(m)]
    fn export(
        py: Python,
        inox_engine: &mut SABIEngine,
        file_to_export: &str,
        load_immediately: bool,
    ) -> PyResult<bool> {
        inox_engine.export(py, file_to_export, load_immediately)
    }

    #[pyfn(m)]
    fn register_nodes(py: Python, inox_engine: &SABIEngine) -> PyResult<bool> {
        inox_engine.register_nodes(py)
    }
    Ok(())
}
