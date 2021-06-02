use std::path::{Path, PathBuf};

pub const DATA_FOLDER: &str = "./data/";

pub trait Data {
    #[inline]
    fn get_data_folder(&self) -> PathBuf {
        PathBuf::from(DATA_FOLDER)
    }
}

#[inline]
pub fn get_absolute_data_path(path: &Path) -> PathBuf {
    let mut pathbuf = PathBuf::from(DATA_FOLDER);
    let data_folder = pathbuf
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let string = path.to_str().unwrap().to_string();
    if string.contains(DATA_FOLDER) {
        pathbuf = path.canonicalize().unwrap()
    } else if string.contains(data_folder.as_str()) {
        pathbuf = path.to_path_buf()
    } else {
        let result_path = pathbuf.join(path);
        pathbuf = result_path.canonicalize().unwrap()
    }
    debug_assert!(pathbuf.exists() && pathbuf.is_file());
    pathbuf
}
