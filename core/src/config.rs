#![allow(dead_code)]

use crate::resources::data::*;
use std::path::PathBuf;

pub const CONFIG_FOLDER: &str = "config";

pub trait ConfigBase: Data {
    fn get_folder(&self) -> PathBuf {
        self.get_data_folder().join(CONFIG_FOLDER)
    }
    fn get_filepath(&self) -> PathBuf {
        self.get_folder().join(self.get_filename())
    }
    fn get_filename(&self) -> &'static str;
}
