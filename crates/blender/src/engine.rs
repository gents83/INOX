use pyo3::{pyclass, pymethods, PyResult, Python};

use crate::exporter::Exporter;
use crate::node_graph::{register_node, RustNode};

use std::{
    io::Write,
    net::{Shutdown, TcpStream},
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread::{self, JoinHandle},
};

#[derive(Default)]
struct ThreadData {
    can_continue: Arc<AtomicBool>,
    files_to_load: Vec<PathBuf>,
}

unsafe impl Send for ThreadData {}
unsafe impl Sync for ThreadData {}

#[pyclass]
pub struct NRGEngine {
    is_running: Arc<AtomicBool>,
    exporter: Exporter,
    current_dir: PathBuf,
    thread_data: Arc<RwLock<ThreadData>>,
    process: Option<std::process::Child>,
    client_thread: Option<JoinHandle<()>>,
}

impl Default for NRGEngine {
    fn default() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            current_dir: PathBuf::new(),
            thread_data: Arc::new(RwLock::new(ThreadData::default())),
            process: None,
            client_thread: None,
            exporter: Exporter::default(),
        }
    }
}

#[pymethods]
impl NRGEngine {
    #[new]
    fn new() -> Self {
        Self::default()
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
    pub fn start(&mut self, executable_path: &str) -> PyResult<bool> {
        println!("NRGEngine started");

        let mut path = PathBuf::from(executable_path);

        self.current_dir = path.clone();
        if self.current_dir.ends_with("release") || self.current_dir.ends_with("debug") {
            self.current_dir.pop();
            self.current_dir.pop();
            self.current_dir.pop();
        }

        path = path.join("nrg_launcher.exe");

        let mut command = Command::new(path.as_path());
        command
            .arg("-plugin nrg_connector")
            .arg("-plugin nrg_viewer");
        command.current_dir(self.current_dir.as_path());

        if let Ok(process) = command.spawn() {
            self.process = Some(process);
            self.is_running.store(true, Ordering::SeqCst);

            let thread_data = self.thread_data.clone();
            thread_data.write().unwrap().can_continue = self.is_running.clone();

            let builder = thread::Builder::new().name("Blender client thread".to_string());
            let client_thread = builder
                .spawn(move || client_thread_execution(thread_data))
                .unwrap();
            self.client_thread = Some(client_thread);
        }

        Ok(true)
    }

    pub fn stop(&mut self) {
        println!("NRGEngine stopped");
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub fn export(
        &mut self,
        py: Python,
        file_to_export: &str,
        load_immediately: bool,
    ) -> PyResult<bool> {
        let current_dir = self.current_dir.clone();
        let scenes = self.exporter.process(
            py,
            current_dir.as_path(),
            PathBuf::from(file_to_export).as_path(),
        )?;
        if load_immediately {
            for scene_file in scenes {
                self.thread_data
                    .write()
                    .unwrap()
                    .files_to_load
                    .insert(0, PathBuf::from(scene_file));
            }
        }
        Ok(true)
    }

    pub fn register_nodes(&self, py: Python) -> PyResult<bool> {
        println!("Registering nodes");

        register_node::<RustNode>(py)?;

        Ok(true)
    }
}

fn client_thread_execution(thread_data: Arc<RwLock<ThreadData>>) {
    match TcpStream::connect("127.0.0.1:1983") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 1983");
            let is_running = thread_data.read().unwrap().can_continue.clone();
            while is_running.load(Ordering::SeqCst) {
                let file = { thread_data.write().unwrap().files_to_load.pop() };
                if let Some(file) = file {
                    let file = file.to_str().unwrap_or_default().to_string();

                    println!("NRGEngine sending to load {:?}", file);

                    let message = format!("-load_file {}", file);
                    let msg = message.as_bytes();

                    stream.write(msg).unwrap();
                }
                thread::yield_now();
            }
            stream
                .shutdown(Shutdown::Both)
                .expect("Client thread shutdown call failed");
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}
