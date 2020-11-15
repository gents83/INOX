#![allow(bad_style, overflowing_literals, dead_code, unused_unsafe)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

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
    pub library: nrg_platform::library::Library,
}

impl Lib {
    pub fn new() -> Lib {
        let library_path = get_vulkan_lib_path();
        Lib { library: nrg_platform::library::Library::new(library_path) }
    }
}


include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


/*
pub struct Library {
    pub vkAcquireNextImage2KHR: PFN_vkAcquireNextImage2KHR,
    pub vkCmdBlitImage2KHR: PFN_vkCmdBlitImage2KHR,
}

impl<'a> Library {
    pub fn new(lib : &'a Library) -> Library {
        Library {
            vkAcquireNextImage2KHR:
                lib.library.get::<PFN_vkAcquireNextImage2KHR>("vkAcquireNextImage2KHR"),
            vkCmdBlitImage2KHR: 
                lib.library.get::<PFN_vkCmdBlitImage2KHR>("vkCmdBlitImage2KHR"),
        }
    }
}
*/