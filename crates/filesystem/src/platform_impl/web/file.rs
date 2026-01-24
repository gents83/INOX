use std::path::PathBuf;

pub fn get_exe_path() -> PathBuf {
    std::path::PathBuf::from("/")
}
pub fn get_exe_folder() -> PathBuf {
    std::path::PathBuf::from("/")
}
pub fn get_exe_filename() -> String {
    String::from("inox_launcher.wasm")
}
