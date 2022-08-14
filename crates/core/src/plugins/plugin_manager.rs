use std::{
    path::{Path, PathBuf},
    process,
};

use inox_filesystem::{library, Library};
use inox_platform::{FileEvent, FileWatcher};

use crate::{
    ContextRc, PfnCreatePlugin, PfnDestroyPlugin, PfnLoadConfigPlugin, PfnPreparePlugin,
    PfnUnpreparePlugin, PluginHolder, PluginId, CREATE_PLUGIN_FUNCTION_NAME,
    DESTROY_PLUGIN_FUNCTION_NAME, LOAD_CONFIG_PLUGIN_FUNCTION_NAME, PREPARE_PLUGIN_FUNCTION_NAME,
    UNPREPARE_PLUGIN_FUNCTION_NAME,
};

pub static IN_USE_PREFIX: &str = "in_use";
static mut UNIQUE_LIB_INDEX: u32 = 0;

pub struct PluginData {
    lib: Box<Library>,
    pub plugin_holder: Option<PluginHolder>,
    filewatcher: FileWatcher,
    pub original_path: PathBuf,
    in_use_path: PathBuf,
}

unsafe impl Send for PluginData {}
unsafe impl Sync for PluginData {}

#[derive(Default)]
pub struct PluginManager {
    dynamic_plugins: Vec<PluginData>,
    static_plugins: Vec<PluginHolder>,
}

impl PluginManager {
    pub fn release(&mut self) -> (Vec<PluginId>, Vec<PluginId>) {
        let mut dynamic_plugins_to_remove: Vec<PluginId> = Vec::new();
        let mut static_plugins_to_remove: Vec<PluginId> = Vec::new();
        for plugin in self.dynamic_plugins.iter() {
            dynamic_plugins_to_remove.push(plugin.plugin_holder.as_ref().unwrap().id());
        }
        for plugin_holder in self.static_plugins.iter() {
            static_plugins_to_remove.push(plugin_holder.id());
        }
        (dynamic_plugins_to_remove, static_plugins_to_remove)
    }

    pub fn add_static_plugin(&mut self, plugin_holder: PluginHolder) -> PluginId {
        let plugin_id = plugin_holder.id();
        self.static_plugins.push(plugin_holder);
        plugin_id
    }

    pub fn add_dynamic_plugin(&mut self, plugin_data: PluginData) -> PluginId {
        let plugin_id = plugin_data.plugin_holder.as_ref().unwrap().id();
        self.dynamic_plugins.push(plugin_data);
        plugin_id
    }

    pub fn remove_static_plugin(&mut self, plugin_id: &PluginId) -> Option<PluginHolder> {
        if let Some(index) = self
            .static_plugins
            .iter()
            .position(|plugin_holder| plugin_holder.id() == *plugin_id)
        {
            return Some(self.static_plugins.remove(index));
        } else {
            eprintln!("Unable to find requested plugin with id {:?}", plugin_id);
        }
        None
    }

    pub fn remove_dynamic_plugin(&mut self, plugin_id: &PluginId) -> Option<PluginData> {
        if let Some(index) = self
            .dynamic_plugins
            .iter()
            .position(|plugin| plugin.plugin_holder.as_ref().unwrap().id() == *plugin_id)
        {
            return Some(self.dynamic_plugins.remove(index));
        } else {
            eprintln!("Unable to find requested plugin with id {:?}", plugin_id);
        }
        None
    }

    fn compute_dynamic_name(lib_path: &Path) -> PathBuf {
        unsafe {
            let (path, filename) = library::compute_folder_and_filename(lib_path);
            let mut in_use_filename =
                format!("{}__{}_{}_", IN_USE_PREFIX, process::id(), UNIQUE_LIB_INDEX);
            in_use_filename.push_str(filename.to_str().unwrap());
            UNIQUE_LIB_INDEX += 1;
            let in_use_path = path.join(IN_USE_PREFIX);
            if !in_use_path.exists() {
                let res = std::fs::create_dir_all(in_use_path.clone());
                if res.is_err() {
                    eprintln!(
                        "Folder creation failed {:?} - unable to create in_use folder {}",
                        res.err(),
                        in_use_path.to_str().unwrap(),
                    );
                }
            }
            in_use_path.join(in_use_filename)
        }
    }

    fn load_dynamic_plugin(
        fullpath: PathBuf,
        context: &ContextRc,
    ) -> (library::Library, Option<PluginHolder>) {
        let lib = library::Library::new(fullpath.to_str().unwrap());
        if let Some(create_fn) = lib.get::<PfnCreatePlugin>(CREATE_PLUGIN_FUNCTION_NAME) {
            let mut plugin_holder = unsafe { create_fn.unwrap()(context) };
            plugin_holder.destroy_fn = lib
                .get::<PfnDestroyPlugin>(DESTROY_PLUGIN_FUNCTION_NAME)
                .unwrap();
            plugin_holder.load_config_fn = lib
                .get::<PfnLoadConfigPlugin>(LOAD_CONFIG_PLUGIN_FUNCTION_NAME)
                .unwrap();
            plugin_holder.prepare_fn = lib
                .get::<PfnPreparePlugin>(PREPARE_PLUGIN_FUNCTION_NAME)
                .unwrap();
            plugin_holder.unprepare_fn = lib
                .get::<PfnUnpreparePlugin>(UNPREPARE_PLUGIN_FUNCTION_NAME)
                .unwrap();
            return (lib, Some(plugin_holder));
        }
        (lib, None)
    }

    pub fn create_plugin_data(lib_path: &Path, context: &ContextRc) -> PluginData {
        let (path, filename) = library::compute_folder_and_filename(lib_path);
        let fullpath = path.join(filename);
        if !fullpath.exists() && fullpath.is_file() {
            panic!(
                "Unable to find requested plugin path {}",
                fullpath.to_str().unwrap()
            );
        }
        let mut in_use_fullpath = PluginManager::compute_dynamic_name(fullpath.as_path());
        let res = std::fs::copy(fullpath.clone(), in_use_fullpath.clone());
        if res.is_err() {
            eprintln!(
                "Copy failed {:?} - unable to create in_use version of the lib {}\nUsing {}",
                res.err(),
                in_use_fullpath.to_str().unwrap(),
                fullpath.to_str().unwrap(),
            );
            in_use_fullpath = fullpath.clone();
        }

        let (lib, plugin_holder) =
            PluginManager::load_dynamic_plugin(in_use_fullpath.clone(), context);
        /*
        debug_log!(
            "Loaded plugin {}",
            fullpath.file_stem().unwrap().to_str().unwrap(),
        );
        */

        Self::load_config_plugin_holder(plugin_holder.as_ref().unwrap(), context);
        Self::prepare_plugin_holder(plugin_holder.as_ref().unwrap(), context);

        PluginData {
            lib: Box::new(lib),
            plugin_holder,
            filewatcher: FileWatcher::new(fullpath.clone()),
            original_path: fullpath,
            in_use_path: in_use_fullpath,
        }
    }

    pub fn load_config_plugin_holder(plugin_holder: &PluginHolder, context: &ContextRc) {
        if let Some(load_config_fn) = plugin_holder.load_config_fn.as_ref() {
            unsafe { load_config_fn(context) };
        }
    }

    pub fn prepare_plugin_holder(plugin_holder: &PluginHolder, context: &ContextRc) {
        if let Some(prepare_fn) = plugin_holder.prepare_fn.as_ref() {
            unsafe { prepare_fn(context) };
        }
    }

    pub fn release_plugin_holder(plugin_holder: PluginHolder, context: &ContextRc) {
        if let Some(unprepare_fn) = plugin_holder.unprepare_fn.as_ref() {
            unsafe { unprepare_fn(context) };
        }
        if let Some(destroy_fn) = plugin_holder.destroy_fn.as_ref() {
            unsafe { destroy_fn() };
        }
    }

    pub fn clear_plugin_data(mut plugin_data: PluginData, context: &ContextRc) {
        inox_profiler::scoped_profile!("plugin_manager::clear_plugin_data");
        plugin_data.filewatcher.stop();

        let in_use_path = plugin_data.in_use_path;
        let lib = unsafe { Box::into_raw(plugin_data.lib).as_mut().unwrap() };
        if let Some(plugin_holder) = plugin_data.plugin_holder {
            Self::release_plugin_holder(plugin_holder, context);
        }
        lib.close();
        /*
        debug_log!(
            "Unloaded plugin {:?}",
            plugin_data.original_path.as_os_str(),
        );
        */

        delete_file(in_use_path);
    }

    pub fn update(&mut self) -> Vec<PluginId> {
        inox_profiler::scoped_profile!("plugin_manager::update");

        let mut plugins_to_remove: Vec<PluginId> = Vec::new();
        for plugin_data in self.dynamic_plugins.iter_mut() {
            while let Ok(event) = plugin_data.filewatcher.read_events().try_recv() {
                match event {
                    FileEvent::Modified(path)
                    | FileEvent::Created(path)
                    | FileEvent::RenamedTo(path) => {
                        if plugin_data.filewatcher.get_path().eq(&path) {
                            plugins_to_remove
                                .push(plugin_data.plugin_holder.as_ref().unwrap().id());
                        }
                    }
                    _ => {}
                }
            }
        }

        plugins_to_remove
    }

    pub fn get_plugin_data(&mut self, id: PluginId) -> Option<&mut PluginData> {
        if let Some(index) = self
            .dynamic_plugins
            .iter()
            .position(|plugin| plugin.plugin_holder.as_ref().unwrap().id() == id)
        {
            return Some(&mut self.dynamic_plugins[index]);
        }
        None
    }
}
