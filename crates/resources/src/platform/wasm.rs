#![cfg(target_arch = "wasm32")]

use crate::{Data, DATA_FOLDER, DATA_RAW_FOLDER, WEB_FOLDER};
use std::path::PathBuf;

impl Data {
    #[inline]
    pub fn data_raw_folder() -> PathBuf {
        PathBuf::from(".").join(DATA_RAW_FOLDER)
    }
    #[inline]
    pub fn data_folder() -> PathBuf {
        PathBuf::from(".").join(DATA_FOLDER)
    }
    #[inline]
    pub fn platform_data_folder() -> PathBuf {
        PathBuf::from(".").join(DATA_FOLDER).join(WEB_FOLDER)
    }
}
