#![allow(non_camel_case_types)]
extern crate nrg_platform;

pub fn get_core_project_folder_name() -> String {
    String::from("core")
}

#[cfg(windows)]
pub fn get_core_lib_path() -> String {
    String::from("nrg_core.dll")
}
#[cfg(all(unix, not(target_os = "macos")))]
pub fn get_core_lib_path() -> String {
    String::from("libnrg_core.so")
}
#[cfg(target_os = "macos")]
pub fn get_core_lib_path() -> String {
    String::from("libnrg_core.dylib")
}
// Note: this is an opaque type
#[repr(C)]
pub struct Entity {
    pub transform: u32,
}


pub type PFN_create_entity_internal = unsafe extern "C" fn()-> Entity;
pub type PFN_create_entity = ::std::option::Option<unsafe extern "C" fn()-> Entity>;
pub static mut CreateEntity:PFN_create_entity = None;

pub type PFN_create_entity_with_param = ::std::option::Option<unsafe extern "C" fn(_integer:u32)-> Entity>;
pub static mut CreateEntityWithParam:PFN_create_entity_with_param = None;

pub struct CoreLib;
/*/
macro_rules! declare_internal_function {
    ($name:ident, $type:ty) => {    
        pub fn $name() -> $type::ReturnType {
            unsafe {
                $name.unwrap()()
            }
        }
    };
}
*/

impl CoreLib {
    pub fn load() -> Self {
        let lib = nrg_platform::library::Library::new(get_core_lib_path().as_str());
        unsafe {
            CreateEntity = lib.get::<PFN_create_entity>("create_entity");
            CreateEntityWithParam = lib.get::<PFN_create_entity_with_param>("create_entity_with_param");
        }
        Self{}
    }

    //declare_internal_function!(CreateEntity, PFN_create_entity);
}