use std::{collections::HashMap, path::PathBuf};
use std::{env, thread, time};

use nrg_platform::*;

use super::plugin::*;
use crate::resources::shared_data::*;
use crate::schedule::scheduler::*;

pub static IN_USE_PREFIX: &str = "in_use";
static WAIT_TIME_BEFORE_RELOADING: u64 = 500;
static mut UNIQUE_LIB_INDEX: u32 = 0;

struct PluginData {
    lib: Box<Library>,
    plugin_holder: PluginHolder,
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

    pub fn release(&mut self, scheduler: &mut Scheduler) {
        let mut plugins_to_remove: Vec<PluginId> = Vec::new();
        for (id, _p) in self.plugins.iter() {
            plugins_to_remove.push(*id);
        }
        for id in plugins_to_remove.iter() {
            self.remove_plugin(id, scheduler);
        }
        self.plugins.clear();
    }

    pub fn add_plugin(
        &mut self,
        lib_path: PathBuf,
        shared_data: &mut SharedDataRw,
        scheduler: &mut Scheduler,
    ) -> PluginId {
        let mut plugin_data = PluginManager::create_plugin_data(lib_path);
        plugin_data
            .plugin_holder
            .get_plugin()
            .prepare(scheduler, shared_data);
        let id = plugin_data.plugin_holder.get_plugin().id();
        self.plugins.insert(id, plugin_data);
        id
    }

    pub fn remove_plugin(&mut self, plugin_id: &PluginId, scheduler: &mut Scheduler) {
        if self.plugins.contains_key(plugin_id) {
            let in_use_path = {
                let plugin_data = self.plugins.remove_entry(plugin_id).unwrap().1;
                let path = plugin_data.in_use_path.clone();
                PluginManager::clear_plugin_data(plugin_data, scheduler);
                path
            };

            delete_file(in_use_path);
        } else {
            eprintln!("Unable to find requested plugin with id {:?}", plugin_id);
        }
    }

    fn compute_folder_and_filename(lib_path: PathBuf) -> (PathBuf, PathBuf) {
        let mut path = lib_path;
        let mut filename = path.clone();
        if path.is_absolute() {
            filename = PathBuf::from(path.file_name().unwrap());
            path = PathBuf::from(path.parent().unwrap());
        } else {
            path = env::current_exe().unwrap().parent().unwrap().to_path_buf();
        }
        path = path.canonicalize().unwrap();
        (path, filename)
    }

    fn compute_dynamic_name(lib_path: PathBuf) -> PathBuf {
        unsafe {
            let (path, filename) = PluginManager::compute_folder_and_filename(lib_path);
            let mut in_use_filename = format!("{}_{}_", IN_USE_PREFIX, UNIQUE_LIB_INDEX);
            in_use_filename.push_str(filename.to_str().unwrap());
            UNIQUE_LIB_INDEX += 1;
            let in_use_path = path.join(IN_USE_PREFIX);
            if !in_use_path.exists() {
                let res = std::fs::create_dir(in_use_path.clone());
                if res.is_err() {
                    eprintln!(
                        "Folder creation failed {:?} - unable to create in_use folder {}",
                        res.err(),
                        in_use_path.to_str().unwrap()
                    );
                }
            }
            in_use_path.join(in_use_filename)
        }
    }

    fn create_plugin_data(lib_path: PathBuf) -> PluginData {
        let (path, filename) = PluginManager::compute_folder_and_filename(lib_path);
        let fullpath = path.join(filename);
        if !fullpath.exists() {
            panic!(
                "Unable to find requested plugin path {}",
                fullpath.to_str().unwrap()
            );
        }
        let in_use_fullpath = PluginManager::compute_dynamic_name(fullpath.clone());
        let res = std::fs::copy(fullpath.clone(), in_use_fullpath.clone());
        if res.is_err() {
            eprintln!(
                "Copy failed {:?} - unable to create in_use version of the lib {}",
                res.err(),
                in_use_fullpath.to_str().unwrap()
            );
        }

        let (lib, plugin_holder) = PluginManager::load_plugin(in_use_fullpath.clone());
        PluginData {
            lib: Box::new(lib),
            plugin_holder,
            filewatcher: FileWatcher::new(fullpath.clone()),
            original_path: fullpath,
            in_use_path: in_use_fullpath,
        }
    }

    fn load_plugin(fullpath: PathBuf) -> (library::Library, PluginHolder) {
        let lib = library::Library::new(fullpath);
        let create_fn = lib.get::<PFNCreatePlugin>(CREATE_PLUGIN_FUNCTION_NAME);

        let plugin_holder = unsafe { create_fn.unwrap()() };
        (lib, plugin_holder)
    }

    fn clear_plugin_data(mut plugin_data: PluginData, scheduler: &mut Scheduler) {
        plugin_data.filewatcher.stop();
        thread::sleep(time::Duration::from_millis(WAIT_TIME_BEFORE_RELOADING));

        plugin_data.plugin_holder.get_plugin().unprepare(scheduler);

        let lib = unsafe { Box::into_raw(plugin_data.lib).as_mut().unwrap() };
        let destroy_fn = lib.get::<PFNDestroyPlugin>(DESTROY_PLUGIN_FUNCTION_NAME);
        unsafe { destroy_fn.unwrap()(plugin_data.plugin_holder) };
        lib.close();

        thread::sleep(time::Duration::from_millis(WAIT_TIME_BEFORE_RELOADING));
    }

    pub fn update(&mut self, shared_data: &mut SharedDataRw, scheduler: &mut Scheduler) {
        let mut plugins_to_remove: Vec<PluginId> = Vec::new();
        for (id, plugin_data) in self.plugins.iter_mut() {
            if let Ok(FileEvent::Modified(path)) = plugin_data.filewatcher.read_events().try_recv()
            {
                if plugin_data.filewatcher.get_path().eq(&path) {
                    plugins_to_remove.push(*id);
                }
            }
        }
        for id in plugins_to_remove.iter() {
            let lib_path = self.plugins[id].original_path.clone();
            self.remove_plugin(id, scheduler);
            self.add_plugin(lib_path, shared_data, scheduler);
        }
    }
}
