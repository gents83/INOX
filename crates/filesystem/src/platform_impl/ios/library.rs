use std::path::PathBuf;

pub fn library_filename(name: &str) -> String {
    format!("lib{}.dylib", name)
}

pub fn library_path() -> PathBuf {
    let path = std::env::current_exe().unwrap();
    let path = path.parent().unwrap();
    path.to_path_buf()
}

pub struct Library;
impl Library {
    pub fn load(_filename: &str) -> Option<Self> {
        None
    }
    pub fn get<T>(&self, _symbol: &str) -> Option<T> {
        None
    }
    pub fn close(&self) {}
}
