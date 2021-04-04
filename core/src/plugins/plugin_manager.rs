use std::env;
use std::path::PathBuf;

use nrg_platform::{delete_file, library, FileEvent, FileWatcher, Library};

use crate::{
    PFNCreatePlugin, PFNDestroyPlugin, PluginHolder, PluginId, Scheduler, SharedDataRw,
    CREATE_PLUGIN_FUNCTION_NAME, DESTROY_PLUGIN_FUNCTION_NAME,
};

pub static IN_USE_PREFIX: &str = "in_use";
static mut UNIQUE_LIB_INDEX: u32 = 0;

struct PluginData {
    id: PluginId,
    lib: Box<Library>,
    plugin_holder: Option<PluginHolder>,
    filewatcher: FileWatcher,
    original_path: PathBuf,
    in_use_path: PathBuf,
}

pub struct PluginManager {
    plugins: Vec<PluginData>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn release(&mut self, scheduler: &mut Scheduler) {
        let mut plugins_to_remove: Vec<PluginId> = Vec::new();
        for plugin in self.plugins.iter() {
            plugins_to_remove.push(plugin.id);
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
    ) {
        let mut plugin_data = PluginManager::create_plugin_data(lib_path);
        if let Some(plugin_holder) = &mut plugin_data.plugin_holder {
            plugin_holder.get_plugin().prepare(scheduler, shared_data);
        }
        self.plugins.push(plugin_data);
    }

    pub fn remove_plugin(&mut self, plugin_id: &PluginId, scheduler: &mut Scheduler) {
        if let Some(index) = self.plugins.iter().position(|el| el.id == *plugin_id) {
            let in_use_path = {
                let plugin_data = self.plugins.remove(index);
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
                let res = std::fs::create_dir_all(in_use_path.clone());
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

        println!("Loaded plugin {:?}", fullpath.as_os_str());

        PluginData {
            id: PluginId::new(),
            lib: Box::new(lib),
            plugin_holder,
            filewatcher: FileWatcher::new(fullpath.clone()),
            original_path: fullpath,
            in_use_path: in_use_fullpath,
        }
    }

    fn load_plugin(fullpath: PathBuf) -> (library::Library, Option<PluginHolder>) {
        let lib = library::Library::new(fullpath);
        if let Some(create_fn) = lib.get::<PFNCreatePlugin>(CREATE_PLUGIN_FUNCTION_NAME) {
            let plugin_holder = unsafe { create_fn.unwrap()() };
            return (lib, Some(plugin_holder));
        }
        (lib, None)
    }

    fn clear_plugin_data(mut plugin_data: PluginData, scheduler: &mut Scheduler) {
        plugin_data.filewatcher.stop();

        let lib = unsafe { Box::into_raw(plugin_data.lib).as_mut().unwrap() };
        if let Some(mut plugin_holder) = plugin_data.plugin_holder {
            plugin_holder.get_plugin().unprepare(scheduler);
            if let Some(destroy_fn) = lib.get::<PFNDestroyPlugin>(DESTROY_PLUGIN_FUNCTION_NAME) {
                unsafe { destroy_fn.unwrap()(plugin_holder) };
            }
        }
        lib.close();

        println!(
            "Unloaded plugin {:?}",
            plugin_data.original_path.as_os_str()
        );
    }

    pub fn update(&mut self, shared_data: &mut SharedDataRw, scheduler: &mut Scheduler) {
        let mut plugins_to_remove: Vec<PluginId> = Vec::new();
        for plugin_data in self.plugins.iter_mut() {
            if let Ok(FileEvent::Modified(path)) = plugin_data.filewatcher.read_events().try_recv()
            {
                if plugin_data.filewatcher.get_path().eq(&path) {
                    plugins_to_remove.push(plugin_data.id);
                }
            }
        }
        for id in plugins_to_remove.iter() {
            if let Some(index) = self.plugins.iter().position(|el| el.id == *id) {
                let lib_path = self.plugins[index].original_path.clone();
                self.remove_plugin(id, scheduler);
                self.add_plugin(lib_path, shared_data, scheduler);
            }
        }
    }
}
