
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