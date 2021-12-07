use crate::exporter::Exporter;
use pyo3::{pyclass, pymethods, PyResult, Python};
use sabi_resources::Singleton;

use sabi_binarizer::Binarizer;
use sabi_core::App;
use sabi_nodes::{LogicNodeRegistry, NodeType};
use sabi_resources::{DATA_FOLDER, DATA_RAW_FOLDER};

use std::{
    env,
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
pub struct SABIEngine {
    is_running: Arc<AtomicBool>,
    exporter: Exporter,
    binarizer: Binarizer,
    app: App,
    app_dir: PathBuf,
    working_dir: PathBuf,
    thread_data: Arc<RwLock<ThreadData>>,
    process: Option<std::process::Child>,
    client_thread: Option<JoinHandle<()>>,
}

#[pymethods]
impl SABIEngine {
    #[new]
    fn new(executable_path: &str, plugins_to_load: Vec<String>) -> Self {
        let app_dir = PathBuf::from(executable_path);

        let mut working_dir = app_dir.clone();
        if working_dir.ends_with("release") || working_dir.ends_with("debug") {
            working_dir.pop();
            working_dir.pop();
            working_dir.pop();
        }
        env::set_current_dir(&working_dir).ok();

        let mut app = App::default();

        let mut binarizer = Binarizer::new(
            app.get_global_messenger(),
            working_dir.join(DATA_RAW_FOLDER),
            working_dir.join(DATA_FOLDER),
        );
        binarizer.stop();

        plugins_to_load.iter().for_each(|plugin| {
            let mut plugin_path = app_dir.clone();
            plugin_path = plugin_path.join(plugin);
            println!("Loading plugin: {:?}", plugin_path);
            app.add_plugin(plugin_path);
        });

        Self {
            app_dir,
            working_dir,
            is_running: Arc::new(AtomicBool::new(false)),
            thread_data: Arc::new(RwLock::new(ThreadData::default())),
            process: None,
            client_thread: None,
            exporter: Exporter::default(),
            binarizer,
            app,
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
    pub fn start(&mut self) -> PyResult<bool> {
        println!("SABIEngine started");

        let path = self.app_dir.join("sabi_launcher.exe");

        let mut command = Command::new(path.as_path());
        command
            .arg("-plugin sabi_connector")
            .arg("-plugin sabi_viewer");
        command.current_dir(self.working_dir.as_path());

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
        println!("SABIEngine stopped");
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub fn export(
        &mut self,
        py: Python,
        file_to_export: &str,
        load_immediately: bool,
    ) -> PyResult<bool> {
        let current_dir = self.working_dir.clone();
        let scenes = self.exporter.process(
            py,
            current_dir.as_path(),
            PathBuf::from(file_to_export).as_path(),
        )?;

        self.binarizer.start();
        while !self.binarizer.is_running() {
            thread::yield_now();
        }
        self.binarizer.stop();

        if load_immediately {
            for scene_file in scenes {
                self.thread_data
                    .write()
                    .unwrap()
                    .files_to_load
                    .insert(0, scene_file);
            }
        }
        Ok(true)
    }

    pub fn register_nodes(&self, py: Python) -> PyResult<bool> {
        let data = self.app.get_shared_data();

        let registry = LogicNodeRegistry::get(data);

        registry.for_each_node(|node| add_node_in_blender(node, py));
        Ok(true)
    }
}

fn add_node_in_blender(node: &dyn NodeType, py: Python) {
    let node_name = node.name();
    let category = node.category();
    let base_class = "LogicNodeBase";
    let description = node.description();
    let serialized_class = node.serialize_node();

    println!("Registering node {}", node_name);

    py.import("SABI")
        .unwrap()
        .getattr("node_tree")
        .unwrap()
        .call_method1(
            "create_node_from_data",
            (
                node_name,
                base_class,
                category,
                description,
                serialized_class,
            ),
        )
        .ok();
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

                    println!("SABIEngine sending to load {:?}", file);

                    let message = format!("-load_file {}", file);
                    let msg = message.as_bytes();

                    stream.write_all(msg).ok();
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
