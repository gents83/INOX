use std::path::{Path, PathBuf};

pub const DATA_RAW_FOLDER: &str = "./data_raw/";
pub const DATA_FOLDER: &str = "./data/";

pub trait Data {
    #[inline]
    fn get_data_folder(&self) -> PathBuf {
        PathBuf::from(DATA_FOLDER)
    }
}

#[inline]
pub fn get_absolute_path_from(parent_folder: &Path, relative_path: &Path) -> PathBuf {
    let mut pathbuf = parent_folder.to_path_buf();
    let data_folder = pathbuf
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let string = relative_path.to_str().unwrap().to_string();
    if string.contains(parent_folder.to_str().unwrap()) {
        pathbuf = relative_path.canonicalize().unwrap()
    } else if string.contains(data_folder.as_str()) {
        pathbuf = relative_path.to_path_buf()
    } else if let Ok(result_path) = pathbuf.join(relative_path).canonicalize() {
        pathbuf = result_path;
    } else {
        eprintln!("Unable to join {:?} with {:?}", pathbuf, relative_path);
    }
    pathbuf
}
