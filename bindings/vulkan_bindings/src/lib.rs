#![allow(bad_style, overflowing_literals, dead_code, unused_unsafe)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(clippy::all)]

#[cfg(windows)]
pub fn get_vulkan_lib_path() -> &'static str {
    "vulkan-1.dll"
}
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub fn get_vulkan_lib_path() -> &'static str {
    "libvulkan.so.1"
}
#[cfg(target_os = "macos")]
pub fn get_vulkan_lib_path() -> &'static str {
    "libvulkan.1.dylib"
}
#[cfg(target_os = "android")]
pub fn get_vulkan_lib_path() -> &'static str {
    "libvulkan.so"
}

pub struct Lib {
    pub library: nrg_filesystem::library::Library,
}

impl Default for Lib {
    fn default() -> Self {
        let library_path = get_vulkan_lib_path();
        Self {
            library: nrg_filesystem::library::Library::new(library_path),
        }
    }
}

#[allow(clippy::all)]
#[allow(unaligned_references)]
pub mod vulkan_generated {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
pub use vulkan_generated::*;
