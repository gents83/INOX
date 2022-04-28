use std::any::Any;

use inox_uid::{generate_uid_from_string, Uid};

use crate::ContextRc;

pub const CREATE_PLUGIN_FUNCTION_NAME: &str = "create_plugin";
pub type PfnCreatePlugin = ::std::option::Option<unsafe fn(context: &ContextRc) -> PluginHolder>;
pub const DESTROY_PLUGIN_FUNCTION_NAME: &str = "destroy_plugin";
pub type PfnDestroyPlugin = ::std::option::Option<unsafe fn()>;
pub const LOAD_CONFIG_PLUGIN_FUNCTION_NAME: &str = "load_config_plugin";
pub type PfnLoadConfigPlugin = ::std::option::Option<unsafe fn(context: &ContextRc)>;
pub const PREPARE_PLUGIN_FUNCTION_NAME: &str = "prepare_plugin";
pub type PfnPreparePlugin = ::std::option::Option<unsafe fn(context: &ContextRc)>;
pub const UNPREPARE_PLUGIN_FUNCTION_NAME: &str = "unprepare_plugin";
pub type PfnUnpreparePlugin = ::std::option::Option<unsafe fn(context: &ContextRc)>;

pub type PluginId = Uid;

pub trait Plugin: Any + Send + Sync {
    fn create(context: &ContextRc) -> Self;
    fn prepare(&mut self, context: &ContextRc);
    fn unprepare(&mut self, context: &ContextRc);
    fn load_config(&mut self, context: &ContextRc);
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
    pub load_config_fn: PfnLoadConfigPlugin,
    pub unprepare_fn: PfnUnpreparePlugin,
}

impl PluginHolder {
    pub fn new(plugin_id: PluginId, name: &str) -> Self {
        Self {
            plugin_id,
            plugin_name: name.to_string(),
            destroy_fn: None,
            prepare_fn: None,
            load_config_fn: None,
            unprepare_fn: None,
        }
    }
    pub fn id(&self) -> PluginId {
        self.plugin_id
    }
}

#[macro_export]
macro_rules! define_dynamic_plugin {
    ($Type:ident) => {
        #[no_mangle]
        pub extern "C" fn create_plugin(context: &$crate::ContextRc) -> $crate::PluginHolder {
            static_plugin::create_plugin(context)
        }

        #[no_mangle]
        pub extern "C" fn destroy_plugin() {
            static_plugin::destroy_plugin()
        }

        #[no_mangle]
        pub extern "C" fn load_config_plugin(context: &$crate::ContextRc) {
            static_plugin::load_config_plugin(context)
        }

        #[no_mangle]
        pub extern "C" fn prepare_plugin(context: &$crate::ContextRc) {
            static_plugin::prepare_plugin(context)
        }

        #[no_mangle]
        pub extern "C" fn unprepare_plugin(context: &$crate::ContextRc) {
            static_plugin::unprepare_plugin(context)
        }
    };
}

#[macro_export]
macro_rules! define_static_plugin {
    ($Type:ident) => {
        pub mod static_plugin {
            use $crate::Plugin;

            pub(crate) static mut PLUGIN: Option<crate::$Type> = None;

            pub fn create_plugin(context: &$crate::ContextRc) -> $crate::PluginHolder {
                let plugin = unsafe { PLUGIN.get_or_insert(crate::$Type::create(context)) };
                $crate::PluginHolder::new(plugin.id(), plugin.name())
            }

            pub fn destroy_plugin() {
                unsafe {
                    debug_assert!(
                        PLUGIN.is_some(),
                        "Destroying {:?} plugin never created",
                        PLUGIN.as_ref().unwrap().name()
                    );
                    PLUGIN = None;
                }
            }

            pub fn load_config_plugin(context: &$crate::ContextRc) {
                unsafe {
                    debug_assert!(
                        PLUGIN.is_some(),
                        "Trying to load_config for {:?} plugin never created",
                        PLUGIN.as_ref().unwrap().name()
                    );
                    PLUGIN.as_mut().unwrap().load_config(context);
                }
            }

            pub fn prepare_plugin(context: &$crate::ContextRc) {
                unsafe {
                    debug_assert!(
                        PLUGIN.is_some(),
                        "Trying to prepare {:?} plugin never created",
                        PLUGIN.as_ref().unwrap().name()
                    );
                    PLUGIN.as_mut().unwrap().prepare(context);
                }
            }

            pub fn unprepare_plugin(context: &$crate::ContextRc) {
                unsafe {
                    debug_assert!(
                        PLUGIN.is_some(),
                        "Trying to unprepare {:?} plugin never created",
                        PLUGIN.as_ref().unwrap().name()
                    );
                    PLUGIN.as_mut().unwrap().unprepare(context);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! define_plugin {
    ($Type:ident) => {
        $crate::define_static_plugin!($Type);

        #[cfg(not(target_arch = "wasm32"))]
        $crate::define_dynamic_plugin!($Type);
    };
}
