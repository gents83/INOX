use std::{env, thread, time};
use std::{collections::HashMap, path::PathBuf};

use nrg_platform::*;

use super::plugin::*;
use super::scheduler::*;
use super::shared_data::*;

static IN_USE_PREFIX:&str = "in_use";
static WAIT_TIME_BEFORE_RELOADING: u64 = 500;
static mut UNIQUE_LIB_INDEX:u32 = 0;

struct PluginData {
    lib: library::Library,
    plugin: Box<dyn Plugin>,
    filewatcher: FileWatcher,
    original_path: PathBuf,
    in_use_path: PathBuf,
}


pub struct PluginManager {
    plugins: HashMap<PluginId, PluginData>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn release(&mut self, shared_data: &mut SharedData, scheduler: &mut Scheduler) {
        self.plugins.retain(|_id, plugin_data| {
            PluginManager::clear_plugin_data(plugin_data, shared_data, scheduler);
            false
        });
    }

    pub fn add_plugin(&mut self, lib_path: PathBuf, shared_data: &mut SharedData, scheduler: &mut Scheduler) -> PluginId {
        let mut plugin_data = PluginManager::create_plugin_data(lib_path.clone());
        plugin_data.plugin.prepare(scheduler, shared_data);
        
        let id = plugin_data.plugin.id();
        self.plugins.insert(id, plugin_data);
        id
    }

    pub fn remove_plugin(&mut self, plugin_id: &PluginId, shared_data: &mut SharedData, scheduler: &mut Scheduler) {
        if self.plugins.contains_key(plugin_id) {
            PluginManager::clear_plugin_data( self.plugins.get_mut(plugin_id).unwrap(), shared_data, scheduler);
            self.plugins.remove(plugin_id);
        }
        else {
            eprintln!("Unable to find requested plugin with id {:?}", plugin_id);
        }
    }

    fn compute_folder_and_filename(lib_path: PathBuf) -> (PathBuf, PathBuf) {
        let mut path = lib_path;
        let mut filename = path.clone();
        
        if path.is_absolute() {
            filename = PathBuf::from(path.file_name().unwrap());
            path = PathBuf::from(path.parent().unwrap());
        }
        else {
            path = env::current_exe().unwrap().parent().unwrap().to_path_buf();
        }
        path = path.canonicalize().unwrap();
        (path, filename)
    }

    fn compute_dynamic_name(lib_path:PathBuf) -> PathBuf {
        unsafe {
            let (path, filename) = PluginManager::compute_folder_and_filename(lib_path);
            let mut in_use_filename = format!("{}_{}_", IN_USE_PREFIX, UNIQUE_LIB_INDEX);
            in_use_filename.push_str(filename.to_str().unwrap());
            UNIQUE_LIB_INDEX += 1;
            path.join(in_use_filename)
        }
    }
    

    fn create_plugin_data(lib_path: PathBuf) -> PluginData {
        let (path, filename) = PluginManager::compute_folder_and_filename(lib_path);
        let fullpath = path.join(filename.clone());
        if !fullpath.exists() {
            panic!("Unable to find requested plugin path {}", fullpath.to_str().unwrap());
        }
        let in_use_fullpath = PluginManager::compute_dynamic_name(fullpath.clone());
        let res = std::fs::copy(fullpath.clone(), in_use_fullpath.clone());
        if !res.is_ok() {
            println!("Copy failed {:?} - unable to create in_use version of the lib {}", res.err(), in_use_fullpath.to_str().unwrap());
        }

        let (lib, plugin) = PluginManager::load_plugin(in_use_fullpath.clone());

        PluginData {
            lib,
            plugin,
            filewatcher: FileWatcher::new(fullpath.clone()),
            original_path: fullpath,
            in_use_path: in_use_fullpath,
        }
    }

    fn load_plugin(fullpath: PathBuf) -> (library::Library, Box<dyn Plugin>) {
        let lib = library::Library::new(fullpath.clone());
        let create_fn = lib.get::<PFNCreatePlugin>(CREATE_PLUGIN_FUNCTION_NAME);
        
        let plugin = unsafe { Box::from_raw(create_fn.unwrap()() ) };
        (lib, plugin)
    }

    fn clear_plugin_data(plugin_data:&mut PluginData, shared_data: &mut SharedData, scheduler: &mut Scheduler) {
        let in_use_path = plugin_data.in_use_path.clone();
        plugin_data.filewatcher.stop();
        
        thread::sleep(time::Duration::from_millis(WAIT_TIME_BEFORE_RELOADING));

        plugin_data.plugin.unprepare(scheduler, shared_data);
        plugin_data.lib.close();

        thread::sleep(time::Duration::from_millis(WAIT_TIME_BEFORE_RELOADING));

        let res = std::fs::remove_file(in_use_path.clone());
        if !res.is_ok() {
            println!("Remove failed {:?} - unable to remove {}", res.err(), in_use_path.to_str().unwrap());
        }
    }

    pub fn update(&mut self, shared_data: &mut SharedData, scheduler: &mut Scheduler) {
        let mut plugins_to_remove: Vec<PluginId> = Vec::new();
        for (id, plugin_data) in self.plugins.iter_mut() {
            if let Ok(event) = plugin_data.filewatcher.read_events().try_recv() {               
                match event {
                    FileEvent::Modified(path) => {
                        if plugin_data.filewatcher.get_path().eq(&path) {
                            plugins_to_remove.push(*id);
                        }
                    },
                    _ => {},
                }
            }
        }
        for id in plugins_to_remove.iter() {
            let lib_path = self.plugins[id].original_path.clone();
            self.remove_plugin(id, shared_data, scheduler);
            self.add_plugin(lib_path, shared_data, scheduler);
        }
    }
}


