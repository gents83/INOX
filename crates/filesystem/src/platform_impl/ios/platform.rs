#![cfg(target_os = "ios")]

pub mod library;
pub mod utils;

pub fn get_exe_path() -> std::path::PathBuf {
    std::path::PathBuf::from("")
}
pub fn get_exe_folder() -> std::path::PathBuf {
    std::path::PathBuf::from("")
}
pub fn get_library_extension() -> &'static str {
    "dylib"
}
pub fn get_copy_command() -> &'static str {
    "cp"
}
