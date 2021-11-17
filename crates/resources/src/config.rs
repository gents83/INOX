#![allow(dead_code)]

use std::path::PathBuf;

use sabi_filesystem::convert_from_local_path;

use crate::{Data, DATA_FOLDER};

pub const CONFIG_FOLDER: &str = "config";

pub trait ConfigBase: Data {
    #[inline]
    fn get_folder(&self) -> PathBuf {
        self.get_data_folder().join(CONFIG_FOLDER)
    }
    #[inline]
    fn get_filepath(&self, plugin_name: &str) -> PathBuf {
        convert_from_local_path(
            PathBuf::from(DATA_FOLDER).as_path(),
            self.get_folder()
                .join(plugin_name)
                .join(self.get_filename())
                .as_path(),
        )
    }
    fn get_filename(&self) -> &'static str;
}
