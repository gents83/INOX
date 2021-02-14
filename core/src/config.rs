#![allow(dead_code)]

use crate::resources::data::*;
use std::path::PathBuf;

pub const CONFIG_FOLDER: &str = "config";

pub trait ConfigBase: Data {
    fn get_folder(&self) -> PathBuf {
        self.get_data_folder().join(CONFIG_FOLDER)
    }
}
