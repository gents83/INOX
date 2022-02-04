#![allow(dead_code)]

use std::path::PathBuf;

use inox_filesystem::convert_from_local_path;

use crate::Data;

pub const CONFIG_FOLDER: &str = "config";

pub trait ConfigBase {
    #[inline]
    fn get_folder(&self) -> PathBuf {
        Data::data_folder().join(CONFIG_FOLDER)
    }
    #[inline]
    fn get_filepath(&self, plugin_name: &str) -> PathBuf {
        convert_from_local_path(
            Data::data_folder().as_path(),
            self.get_folder()
                .join(plugin_name)
                .join(self.get_filename())
                .as_path(),
        )
    }
    fn get_filename(&self) -> &'static str;
}
