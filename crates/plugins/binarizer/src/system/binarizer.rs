use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use inox_core::{ContextRc, System, SystemId, SystemUID};
use inox_messenger::MessageHubRc;

use inox_platform::PlatformType;
use inox_resources::{ConfigBase, SharedDataRc};
use inox_serialize::read_from_file;
use inox_uid::generate_uid_from_string;

use crate::{
    config::Config, CopyCompiler, DataWatcher, FontCompiler, GltfCompiler, ImageCompiler,
    ShaderCompiler,
};

struct Info {
    should_end_on_completion: AtomicBool,
    optimize_meshes: AtomicBool,
}

pub struct Binarizer<const PLATFORM_TYPE: PlatformType> {
    config: Config,
    data_raw_folder: PathBuf,
    data_folder: PathBuf,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    thread_handle: Option<JoinHandle<bool>>,
    is_running: Arc<AtomicBool>,
    is_ready: Arc<AtomicBool>,
    info: Arc<Info>,
}

impl<const PLATFORM_TYPE: PlatformType> Binarizer<PLATFORM_TYPE> {
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
            config: Config::default(),
            shared_data: app_context.shared_data().clone(),
            message_hub: app_context.message_hub().clone(),
            data_raw_folder,
            data_folder,
            thread_handle: None,
            is_running: Arc::new(AtomicBool::new(false)),
            info: Arc::new(Info {
                should_end_on_completion: AtomicBool::new(true),
                optimize_meshes: AtomicBool::new(true),
            }),
            is_ready: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    fn init_binarizer(
        mut binarizer: DataWatcher,
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        data_raw_folder: &Path,
        data_folder: &Path,
        info: &Info,
    ) -> DataWatcher {
        let shader_compiler = ShaderCompiler::<PLATFORM_TYPE>::new(
            shared_data.clone(),
            message_hub.clone(),
            data_raw_folder,
            data_folder,
        );
        let font_compiler = FontCompiler::new(message_hub.clone(), data_raw_folder, data_folder);
        let image_compiler = ImageCompiler::new(message_hub.clone(), data_raw_folder, data_folder);
        let gltf_compiler = GltfCompiler::new(
            shared_data.clone(),
            data_raw_folder,
            data_folder,
            info.optimize_meshes.load(Ordering::SeqCst),
        );
        binarizer.add_handler(shader_compiler);
        binarizer.add_handler(font_compiler);
        binarizer.add_handler(image_compiler);
        binarizer.add_handler(gltf_compiler);
        binarizer
    }

    pub fn start(&mut self) {
        inox_log::debug_log!("Starting data binarizer");
        let mut binarizer = DataWatcher::new(self.data_raw_folder.clone());
        let config_compiler = CopyCompiler::new(
            self.message_hub.clone(),
            self.data_raw_folder.as_path(),
            self.data_folder.as_path(),
        );
        binarizer.add_handler(config_compiler);

        self.is_running.store(true, Ordering::SeqCst);
        let can_continue = self.is_running.clone();
        let is_ready = self.is_ready.clone();
        let info = self.info.clone();
        let builder = thread::Builder::new().name("Data Binarizer".to_string());
        let shared_data = self.shared_data.clone();
        let message_hub = self.message_hub.clone();
        let data_raw_folder = self.data_raw_folder.clone();
        let data_folder = self.data_folder.clone();

        let t = builder
            .spawn(move || -> bool {
                binarizer.binarize_all();
                while !is_ready.load(Ordering::SeqCst) {
                    thread::yield_now();
                }
                let mut binarizer = Self::init_binarizer(
                    binarizer,
                    &shared_data,
                    &message_hub,
                    &data_raw_folder,
                    &data_folder,
                    &info,
                );
                binarizer.binarize_all();

                loop {
                    binarizer.update();
                    thread::yield_now();

                    if info.should_end_on_completion.load(Ordering::SeqCst) {
                        can_continue.store(false, Ordering::SeqCst);
                    }
                    if !can_continue.load(Ordering::SeqCst) {
                        inox_log::debug_log!("Ending data binarizer thread");
                        return false;
                    }
                }
            })
            .unwrap();
        self.thread_handle = Some(t);
    }
    pub fn stop(&mut self) {
        if self.thread_handle.is_some() {
            inox_log::debug_log!("Stopping data binarizer");
            let t = self.thread_handle.take().unwrap();

            self.is_running.store(false, Ordering::SeqCst);
            t.join().unwrap();

            self.thread_handle = None;
        }
    }
}

impl<const PLATFORM_TYPE: PlatformType> SystemUID for Binarizer<PLATFORM_TYPE> {
    fn system_id() -> SystemId
    where
        Self: Sized,
    {
        let mut string = std::any::type_name::<Self>().to_string();
        string.push('_');
        string.push_str(&PLATFORM_TYPE.to_string());
        generate_uid_from_string(&string)
    }
}

impl<const PLATFORM_TYPE: PlatformType> System for Binarizer<PLATFORM_TYPE> {
    fn read_config(&mut self, plugin_name: &str) {
        let info = self.info.clone();
        let is_ready = self.is_ready.clone();
        let file_read_success = read_from_file(
            self.config.get_filepath(plugin_name).as_path(),
            self.shared_data.serializable_registry(),
            Box::new(move |data: Config| {
                inox_log::debug_log!(
                    "Binarizer config says that end on completion is: {}",
                    data.end_on_completion
                );
                info.optimize_meshes
                    .store(data.optimize_meshes, Ordering::SeqCst);
                info.should_end_on_completion
                    .store(data.end_on_completion, Ordering::SeqCst);
                is_ready.store(true, Ordering::SeqCst);
            }),
        );
        if !file_read_success {
            self.is_ready.store(true, Ordering::SeqCst);
        }
    }
    fn should_run_when_not_focused(&self) -> bool {
        true
    }

    fn init(&mut self) {
        self.start();
    }

    fn run(&mut self) -> bool {
        let result = self.is_running();
        if !result {
            self.stop();
            return false;
        }
        result
    }
    fn uninit(&mut self) {
        self.stop();
    }
}
