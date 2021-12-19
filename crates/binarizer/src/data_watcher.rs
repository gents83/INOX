use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use crate::{CopyCompiler, FontCompiler, GltfCompiler, ImageCompiler, ShaderCompiler};
use sabi_filesystem::convert_from_local_path;
use sabi_messenger::MessengerRw;
use sabi_platform::{FileEvent, FileWatcher};
use sabi_resources::SharedDataRc;

pub trait ExtensionHandler {
    fn on_changed(&mut self, path: &Path);
}

pub struct Binarizer {
    data_raw_folder: PathBuf,
    data_folder: PathBuf,
    global_messenger: MessengerRw,
    shared_data: SharedDataRc,
    thread_handle: Option<JoinHandle<bool>>,
    is_running: Arc<AtomicBool>,
}

pub struct DataWatcher {
    filewatcher: FileWatcher,
    handlers: Vec<Box<dyn ExtensionHandler>>,
    data_raw_folder: PathBuf,
    data_folder: PathBuf,
}

unsafe impl Send for DataWatcher {}
unsafe impl Sync for DataWatcher {}

impl Binarizer {
    pub fn new(
        global_messenger: &MessengerRw,
        shared_data: &SharedDataRc,
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
            global_messenger: global_messenger.clone(),
            shared_data: shared_data.clone(),
            data_raw_folder,
            data_folder,
            thread_handle: None,
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn start(&mut self) {
        let mut binarizer = DataWatcher {
            filewatcher: FileWatcher::new(self.data_raw_folder.clone()),
            handlers: Vec::new(),
            data_raw_folder: self.data_raw_folder.clone(),
            data_folder: self.data_folder.clone(),
        };

        let shader_compiler = ShaderCompiler::new(self.global_messenger.clone());
        let config_compiler = CopyCompiler::new(self.global_messenger.clone());
        let font_compiler = FontCompiler::new(self.global_messenger.clone());
        let image_compiler = ImageCompiler::new(self.global_messenger.clone());
        let gltf_compiler =
            GltfCompiler::new(self.global_messenger.clone(), self.shared_data.clone());
        binarizer.add_handler(config_compiler);
        binarizer.add_handler(shader_compiler);
        binarizer.add_handler(font_compiler);
        binarizer.add_handler(image_compiler);
        binarizer.add_handler(gltf_compiler);

        self.is_running.store(false, Ordering::SeqCst);
        let can_continue = self.is_running.clone();
        let builder = thread::Builder::new().name("Data Binarizer".to_string());
        let t = builder
            .spawn(move || -> bool {
                binarizer.binarize_all();
                can_continue.store(true, Ordering::SeqCst);
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

impl DataWatcher {
    pub fn add_handler<H>(&mut self, handler: H)
    where
        H: ExtensionHandler + 'static,
    {
        self.handlers.push(Box::new(handler));
    }

    pub fn update(&mut self) {
        while let Ok(FileEvent::Modified(path)) = self.filewatcher.read_events().try_recv() {
            if path.is_file() {
                self.binarize_file(path.as_path());
            }
        }
    }

    pub fn binarize_all(&mut self) {
        let path = self.data_raw_folder.clone();
        self.binarize_folder(path.as_path());
    }

    fn binarize_file(&mut self, path: &Path) {
        let absolute_path = convert_from_local_path(self.data_raw_folder.as_path(), path);
        for handler in self.handlers.iter_mut() {
            handler.on_changed(absolute_path.as_path());
        }
    }

    fn binarize_folder(&mut self, path: &Path) {
        if let Ok(dir) = std::fs::read_dir(path) {
            dir.for_each(|entry| {
                if let Ok(dir_entry) = entry {
                    let path = dir_entry.path();
                    if !path.is_dir() {
                        self.binarize_file(path.as_path());
                    } else {
                        self.binarize_folder(path.as_path());
                    }
                }
            });
        }
    }
}

impl Drop for DataWatcher {
    fn drop(&mut self) {
        self.filewatcher.stop();
    }
}
