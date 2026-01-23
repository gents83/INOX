use std::path::PathBuf;

pub fn get_exe_path() -> PathBuf {
    std::env::current_exe().unwrap().parent().unwrap().to_path_buf()
}
pub fn get_exe_folder() -> PathBuf {
    std::env::current_exe().unwrap().parent().unwrap().to_path_buf()
}
pub fn get_exe_filename() -> String {
    std::env::current_exe()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}
