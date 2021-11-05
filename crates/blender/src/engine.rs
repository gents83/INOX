use crate::exporter::Exporter;
use cpython::{py_class, PyResult, Python};
use std::{path::PathBuf, process::Command};

#[derive(Default)]
pub struct NRGEngineData {
    is_running: bool,
    process: Option<std::process::Child>,
    exporter: Exporter,
}

py_class!(pub class NRGEngine |py| {
    @shared data data: NRGEngineData;
    def __new__(_cls) -> PyResult<NRGEngine> {
        NRGEngine::create_instance(py, NRGEngineData::default())
    }
});

impl NRGEngine {
    pub fn is_running(&self, py: Python) -> bool {
        let data = self.data(py).borrow();
        data.is_running
    }
    pub fn start(&self, py: Python, executable_path: &str, file_to_export: &str) -> PyResult<()> {
        println!("NRGEngine started");
        let mut data = self.data(py).borrow_mut();
        data.is_running = true;

        let mut path = PathBuf::from(executable_path);

        let mut current_dir = path.clone();
        if current_dir.ends_with("release") || current_dir.ends_with("debug") {
            current_dir.pop();
            current_dir.pop();
            current_dir.pop();
        }

        let files_to_load = data.exporter.process(
            py,
            current_dir.as_path(),
            PathBuf::from(file_to_export).as_path(),
        )?;

        path = path.join("nrg_launcher.exe");

        let mut command = Command::new(path.as_path());
        command.arg("-plugin nrg_viewer");
        for file in files_to_load {
            command.arg("-load_file").arg(file.to_str().unwrap());
        }
        command.current_dir(current_dir.as_path());

        if let Ok(process) = command.spawn() {
            data.process = Some(process);
        }

        Ok(())
    }

    pub fn stop(&self, py: Python) {
        println!("NRGEngine stopped");
        let mut data = self.data(py).borrow_mut();
        data.is_running = false;
    }
}
