use std::any::Any;

use inox_uid::{generate_uid_from_string, Uid};

use crate::App;

pub const CREATE_PLUGIN_FUNCTION_NAME: &str = "create_plugin";
pub type PfnCreatePlugin = ::std::option::Option<unsafe fn() -> PluginHolder>;
pub const DESTROY_PLUGIN_FUNCTION_NAME: &str = "destroy_plugin";
pub type PfnDestroyPlugin = ::std::option::Option<unsafe fn()>;
pub const PREPARE_PLUGIN_FUNCTION_NAME: &str = "prepare_plugin";
pub type PfnPreparePlugin = ::std::option::Option<unsafe fn(app: &mut App)>;
pub const UNPREPARE_PLUGIN_FUNCTION_NAME: &str = "unprepare_plugin";
pub type PfnUnpreparePlugin = ::std::option::Option<unsafe fn(app: &mut App)>;

pub type PluginId = Uid;

pub trait Plugin: Any + Send + Sync {
    fn prepare(&mut self, app: &mut App);
    fn unprepare(&mut self, app: &mut App);
    fn id(&self) -> PluginId {
        generate_uid_from_string(self.name())
    }
    fn name(&self) -> &str;
}

#[repr(C)]
pub struct PluginHolder {
    plugin_id: PluginId,
    plugin_name: String,
    pub destroy_fn: PfnDestroyPlugin,
    pub prepare_fn: PfnPreparePlugin,
    pub unprepare_fn: PfnUnpreparePlugin,
}

impl PluginHolder {
    pub fn new(plugin_id: PluginId, name: &str) -> Self {
        Self {
            plugin_id,
            plugin_name: name.to_string(),
            destroy_fn: None,
            prepare_fn: None,
            unprepare_fn: None,
        }
    }
    pub fn id(&self) -> PluginId {
        self.plugin_id
    }
}

#[macro_export]
macro_rules! define_plugin {
    ($Type:ident) => {
        pub(crate) static mut PLUGIN: Option<$Type> = None;

        #[no_mangle]
        pub extern "C" fn create_plugin() -> $crate::PluginHolder {
            let plugin = unsafe { PLUGIN.get_or_insert($Type::default()) };
            $crate::PluginHolder::new(plugin.id(), plugin.name())
        }

        #[no_mangle]
        pub extern "C" fn destroy_plugin() {
            unsafe {
                debug_assert!(
                    PLUGIN.is_some(),
                    "Destroying {:?} plugin never created",
                    PLUGIN.as_ref().unwrap().name()
                );
                PLUGIN = None;
            }
        }

        #[no_mangle]
        pub extern "C" fn prepare_plugin(app: &mut App) {
            unsafe {
                debug_assert!(
                    PLUGIN.is_some(),
                    "Trying to prepare {:?} plugin never created",
                    PLUGIN.as_ref().unwrap().name()
                );
                PLUGIN.as_mut().unwrap().prepare(app);
            }
        }

        #[no_mangle]
        pub extern "C" fn unprepare_plugin(app: &mut App) {
            unsafe {
                debug_assert!(
                    PLUGIN.is_some(),
                    "Trying to unprepare {:?} plugin never created",
                    PLUGIN.as_ref().unwrap().name()
                );
                PLUGIN.as_mut().unwrap().unprepare(app);
            }
        }
    };
}
