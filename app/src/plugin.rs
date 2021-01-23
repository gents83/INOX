use std::any::Any;
use nrg_platform::*;
use super::app::*;

pub const CREATE_PLUGIN_FUNCTION_NAME:&str = "create_plugin";
pub type PFNCreatePlugin = ::std::option::Option<unsafe extern "C" fn()-> *mut dyn Plugin>;

pub trait Plugin: Any + Send + Sync {
    fn build(&mut self, main_app: &mut App);
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}


pub fn load_plugin(main_app: &mut App, lib_name: &str) -> Box<dyn Plugin> {
    let lib = library::Library::new(lib_name);
    let create_fn = lib.get::<PFNCreatePlugin>(CREATE_PLUGIN_FUNCTION_NAME);
    let mut plugin = unsafe { Box::from_raw(create_fn.unwrap()() ) };
    plugin.build(main_app);
    plugin
}