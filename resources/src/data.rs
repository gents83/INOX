use std::path::PathBuf;

pub const DATA_FOLDER: &str = "./data/";

pub trait Data {
    fn get_data_folder(&self) -> PathBuf {
        PathBuf::from(DATA_FOLDER)
    }
}
