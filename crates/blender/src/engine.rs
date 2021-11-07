use crate::exporter::Exporter;
use cpython::{py_class, PyResult, Python};
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

pub struct NRGEngineData {
    is_running: Arc<AtomicBool>,
    exporter: Exporter,
    current_dir: PathBuf,
    thread_data: Arc<RwLock<ThreadData>>,
    process: Option<std::process::Child>,
    client_thread: Option<JoinHandle<()>>,
}

impl Default for NRGEngineData {
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

py_class!(pub class NRGEngine |py| {
    @shared data data: NRGEngineData;
    def __new__(_cls) -> PyResult<NRGEngine> {
        NRGEngine::create_instance(py, NRGEngineData::default())
    }
});

impl NRGEngine {
    pub fn is_running(&self, py: Python) -> bool {
        let data = self.data(py).borrow();
        data.is_running.load(Ordering::SeqCst)
    }
    pub fn start(&self, py: Python, executable_path: &str) -> PyResult<bool> {
        println!("NRGEngine started");
        let mut data = self.data(py).borrow_mut();

        let mut path = PathBuf::from(executable_path);

        data.current_dir = path.clone();
        if data.current_dir.ends_with("release") || data.current_dir.ends_with("debug") {
            data.current_dir.pop();
            data.current_dir.pop();
            data.current_dir.pop();
        }

        path = path.join("nrg_launcher.exe");

        let mut command = Command::new(path.as_path());
        command
            .arg("-plugin nrg_connector")
            .arg("-plugin nrg_viewer");
        command.current_dir(data.current_dir.as_path());

        if let Ok(process) = command.spawn() {
            data.process = Some(process);
            data.is_running.store(true, Ordering::SeqCst);

            let thread_data = data.thread_data.clone();
            thread_data.write().unwrap().can_continue = data.is_running.clone();

            let builder = thread::Builder::new().name("Blender client thread".to_string());
            let client_thread = builder
                .spawn(move || match TcpStream::connect("127.0.0.1:1983") {
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
                })
                .unwrap();
            data.client_thread = Some(client_thread);
        }

        Ok(true)
    }

    pub fn stop(&self, py: Python) {
        println!("NRGEngine stopped");
        let data = self.data(py).borrow_mut();
        data.is_running.store(false, Ordering::SeqCst);
    }

    pub fn export(&self, py: Python, file_to_export: &str) -> PyResult<bool> {
        let mut data = self.data(py).borrow_mut();
        let current_dir = data.current_dir.clone();
        let scenes = data.exporter.process(
            py,
            current_dir.as_path(),
            PathBuf::from(file_to_export).as_path(),
        )?;
        for scene_file in scenes {
            data.thread_data
                .write()
                .unwrap()
                .files_to_load
                .insert(0, PathBuf::from(scene_file));
        }
        Ok(true)
    }
}
