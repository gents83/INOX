use std::path::{Path, PathBuf};

use nrg_serialize::deserialize_from_file;

pub const DATA_RAW_FOLDER: &str = "./data_raw/";
pub const DATA_FOLDER: &str = "./data/";

pub trait Data {
    #[inline]
    fn get_data_folder(&self) -> PathBuf {
        PathBuf::from(DATA_FOLDER)
    }
}

#[inline]
pub fn convert_from_local_path(parent_folder: &Path, relative_path: &Path) -> PathBuf {
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

pub fn convert_in_local_path(original_path: &Path, base_path: &Path) -> PathBuf {
    let path = original_path.to_str().unwrap().to_string();
    let path = path.replace(
        PathBuf::from(base_path)
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap(),
        "",
    );
    let mut path = path.replace("\\", "/");
    if path.starts_with('/') {
        path.remove(0);
    }
    PathBuf::from(path)
}

pub fn from_file<T>(filepath: &Path) -> T
where
    T: Default + for<'de> nrg_serialize::Deserialize<'de>,
{
    let path = convert_from_local_path(PathBuf::from(DATA_FOLDER).as_path(), filepath);
    let mut data = T::default();
    deserialize_from_file(&mut data, path);
    data
}
