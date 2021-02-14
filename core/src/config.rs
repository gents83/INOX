use std::path::PathBuf;

pub struct Config {
    data_folder: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_folder: PathBuf::from("./data/"),
        }
    }
}

impl Config {
    pub fn get_data_folder(&self) -> &PathBuf {
        &self.data_folder
    }
}
