#![cfg(target_os = "linux")]

pub mod library;
pub mod utils;

pub fn get_exe_path() -> std::path::PathBuf {
    std::env::current_exe().unwrap()
}
pub fn get_exe_folder() -> std::path::PathBuf {
    std::env::current_exe().unwrap().parent().unwrap().to_path_buf()
}
pub fn get_library_extension() -> &'static str {
    "so"
}
pub fn get_copy_command() -> &'static str {
    "cp"
}
