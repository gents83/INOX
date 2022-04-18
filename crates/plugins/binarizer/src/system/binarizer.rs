use std::{
    fs::create_dir_all,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use inox_core::{ContextRc, System};
use inox_messenger::MessageHubRc;

use inox_resources::SharedDataRc;

use crate::{CopyCompiler, DataWatcher, FontCompiler, GltfCompiler, ImageCompiler, ShaderCompiler};

pub struct Binarizer {
    data_raw_folder: PathBuf,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    thread_handle: Option<JoinHandle<bool>>,
    is_running: Arc<AtomicBool>,
}

impl Binarizer {
    pub fn new(
        app_context: &ContextRc,
        mut data_raw_folder: PathBuf,
        mut data_folder: PathBuf,
    ) -> Self {
        if !data_raw_folder.exists() {
            let result = create_dir_all(data_raw_folder.as_path());
            debug_assert!(result.is_ok());
        }
        if !data_folder.exists() {
            let result = create_dir_all(data_folder.as_path());
            debug_assert!(result.is_ok());
        }
        data_raw_folder = data_raw_folder.canonicalize().unwrap();
        data_folder = data_folder.canonicalize().unwrap();
        debug_assert!(
            data_raw_folder.exists() && data_raw_folder.is_dir() && data_raw_folder.is_absolute()
        );
        debug_assert!(data_folder.exists() && data_folder.is_dir() && data_folder.is_absolute());
        Self {
            shared_data: app_context.shared_data().clone(),
            message_hub: app_context.message_hub().clone(),
            data_raw_folder,
            thread_handle: None,
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn start(&mut self) {
        let mut binarizer = DataWatcher::new(self.data_raw_folder.clone());

        let shader_compiler =
            ShaderCompiler::new(self.shared_data.clone(), self.message_hub.clone());
        let config_compiler = CopyCompiler::new(self.message_hub.clone());
        let font_compiler = FontCompiler::new(self.message_hub.clone());
        let image_compiler = ImageCompiler::new(self.message_hub.clone());
        let gltf_compiler = GltfCompiler::new(self.shared_data.clone());
        binarizer.add_handler(config_compiler);
        binarizer.add_handler(shader_compiler);
        binarizer.add_handler(font_compiler);
        binarizer.add_handler(image_compiler);
        binarizer.add_handler(gltf_compiler);

        self.is_running.store(true, Ordering::SeqCst);
        let can_continue = self.is_running.clone();
        let builder = thread::Builder::new().name("Data Binarizer".to_string());
        let t = builder
            .spawn(move || -> bool {
                binarizer.binarize_all();
                loop {
                    binarizer.update();
                    thread::yield_now();

                    if !can_continue.load(Ordering::SeqCst) {
                        return false;
                    }
                }
            })
            .unwrap();
        self.thread_handle = Some(t);
    }
    pub fn stop(&mut self) {
        if self.thread_handle.is_some() {
            let t = self.thread_handle.take().unwrap();

            self.is_running.store(false, Ordering::SeqCst);
            t.join().unwrap();

            self.thread_handle = None;
        }
    }
}

impl System for Binarizer {
    fn read_config(&mut self, _plugin_name: &str) {}
    fn should_run_when_not_focused(&self) -> bool {
        true
    }

    fn init(&mut self) {
        self.start();
    }

    fn run(&mut self) -> bool {
        if !self.is_running() {
            self.stop();
            return false;
        }
        true
    }
    fn uninit(&mut self) {
        self.stop();
    }
}
