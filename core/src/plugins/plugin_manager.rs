use std::path::PathBuf;

use nrg_dynamic_library::{library, Library};
use nrg_platform::{FileEvent, FileWatcher};

use crate::{
    App, PfnCreatePlugin, PfnDestroyPlugin, PluginHolder, PluginId, CREATE_PLUGIN_FUNCTION_NAME,
    DESTROY_PLUGIN_FUNCTION_NAME,
};

pub static IN_USE_PREFIX: &str = "in_use";
static mut UNIQUE_LIB_INDEX: u32 = 0;

pub struct PluginData {
    id: PluginId,
    lib: Box<Library>,
    pub plugin_holder: Option<PluginHolder>,
    filewatcher: FileWatcher,
    pub original_path: PathBuf,
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

    pub fn release(&mut self) -> Vec<PluginId> {
        let mut plugins_to_remove: Vec<PluginId> = Vec::new();
        for plugin in self.plugins.iter() {
            plugins_to_remove.push(plugin.id);
        }
        plugins_to_remove
    }

    pub fn add_plugin(&mut self, plugin_data: PluginData) {
        nrg_profiler::scoped_profile!("plugin_manager::add_plugin");
        self.plugins.push(plugin_data);
    }

    pub fn remove_plugin(&mut self, plugin_id: &PluginId) -> Option<PluginData> {
        nrg_profiler::scoped_profile!("plugin_manager::remove_plugin");
        if let Some(index) = self.plugins.iter().position(|el| el.id == *plugin_id) {
            return Some(self.plugins.remove(index));
        } else {
            eprintln!("Unable to find requested plugin with id {:?}", plugin_id);
        }
        None
    }

    fn compute_dynamic_name(lib_path: PathBuf) -> PathBuf {
        unsafe {
            let (path, filename) = library::compute_folder_and_filename(lib_path);
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

    fn load_plugin(fullpath: PathBuf) -> (library::Library, Option<PluginHolder>) {
        nrg_profiler::scoped_profile!("plugin_manager::load_plugin");
        let lib = library::Library::new(fullpath);
        if let Some(create_fn) = lib.get::<PfnCreatePlugin>(CREATE_PLUGIN_FUNCTION_NAME) {
            let plugin_holder = unsafe { create_fn.unwrap()() };
            return (lib, Some(plugin_holder));
        }
        (lib, None)
    }

    pub fn create_plugin_data(lib_path: PathBuf, app: &mut App) -> PluginData {
        let (path, filename) = library::compute_folder_and_filename(lib_path);
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

        let mut plugin_data = PluginData {
            id: PluginId::new(),
            lib: Box::new(lib),
            plugin_holder,
            filewatcher: FileWatcher::new(fullpath.clone()),
            original_path: fullpath,
            in_use_path: in_use_fullpath,
        };

        if let Some(holder) = &mut plugin_data.plugin_holder {
            holder.get_plugin().prepare(app);
        }

        plugin_data
    }

    pub fn clear_plugin_data(mut plugin_data: PluginData, app: &mut App) {
        nrg_profiler::scoped_profile!("plugin_manager::clear_plugin_data");
        plugin_data.filewatcher.stop();

        let in_use_path = plugin_data.in_use_path;
        let lib = unsafe { Box::into_raw(plugin_data.lib).as_mut().unwrap() };
        if let Some(mut plugin_holder) = plugin_data.plugin_holder {
            plugin_holder.get_plugin().unprepare(app);
            if let Some(destroy_fn) = lib.get::<PfnDestroyPlugin>(DESTROY_PLUGIN_FUNCTION_NAME) {
                unsafe { destroy_fn.unwrap()(plugin_holder) };
            }
        }
        lib.close();

        println!(
            "Unloaded plugin {:?}",
            plugin_data.original_path.as_os_str()
        );

        delete_file(in_use_path);
    }

    pub fn update(&mut self) -> Vec<PluginId> {
        nrg_profiler::scoped_profile!("plugin_manager::update");

        let mut plugins_to_remove: Vec<PluginId> = Vec::new();
        for plugin_data in self.plugins.iter_mut() {
            if let Ok(FileEvent::Modified(path)) = plugin_data.filewatcher.read_events().try_recv()
            {
                if plugin_data.filewatcher.get_path().eq(&path) {
                    plugins_to_remove.push(plugin_data.id);
                }
            }
        }

        plugins_to_remove
    }

    pub fn get_plugin_data(&mut self, id: PluginId) -> Option<&mut PluginData> {
        if let Some(index) = self.plugins.iter().position(|el| el.id == id) {
            return Some(&mut self.plugins[index]);
        }
        None
    }
}
