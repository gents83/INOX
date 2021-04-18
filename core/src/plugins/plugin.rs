use std::any::Any;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Scheduler, SharedDataRw};

pub const CREATE_PLUGIN_FUNCTION_NAME: &str = "create_plugin";
pub type PfnCreatePlugin = ::std::option::Option<unsafe extern "C" fn() -> PluginHolder>;
pub const DESTROY_PLUGIN_FUNCTION_NAME: &str = "destroy_plugin";
pub type PfnDestroyPlugin =
    ::std::option::Option<unsafe extern "C" fn(plugin_holder: PluginHolder)>;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PluginId(u64);

impl Default for PluginId {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginId {
    pub fn new() -> Self {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as _;
        PluginId(secs)
    }
}

pub trait Plugin: Any + Send + Sync {
    #[no_mangle]
    fn prepare(&mut self, scheduler: &mut Scheduler, shared_data: &mut SharedDataRw);
    #[no_mangle]
    fn unprepare(&mut self, scheduler: &mut Scheduler);
    #[no_mangle]
    fn id(&self) -> PluginId {
        PluginId::new()
    }
    #[no_mangle]
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

#[repr(C)]
pub struct PluginHolder {
    plugin: Box<dyn Plugin>,
}

impl PluginHolder {
    pub fn new(plugin: Box<dyn Plugin>) -> Self {
        Self { plugin }
    }
    pub fn get_plugin(&mut self) -> &mut Box<dyn Plugin> {
        &mut self.plugin
    }
    pub fn get_boxed_plugin<T>(self) -> Box<T> {
        unsafe {
            let ptr = Box::into_raw(self.plugin);
            let val: *mut dyn Plugin = std::mem::transmute(ptr);
            Box::from_raw(std::mem::transmute_copy(&val))
        }
    }
}
