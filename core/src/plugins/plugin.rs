use std::any::Any;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::schedule::scheduler::*;
use crate::resources::shared_data::*;



pub const CREATE_PLUGIN_FUNCTION_NAME:&str = "create_plugin";
pub type PFNCreatePlugin = ::std::option::Option<unsafe extern fn()-> *mut dyn Plugin>;
pub const DESTROY_PLUGIN_FUNCTION_NAME:&str = "destroy_plugin";
pub type PFNDestroyPlugin = ::std::option::Option<unsafe extern fn(ptr: *mut dyn Plugin)>;



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
    fn prepare<'a>(&mut self, scheduler: &mut Scheduler, shared_data: &mut SharedDataRw);
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

