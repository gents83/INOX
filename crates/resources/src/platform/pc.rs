#![cfg(target_os = "windows")]

use std::{env, path::PathBuf};

use crate::{Data, DATA_FOLDER, DATA_RAW_FOLDER, PC_FOLDER};

impl Data {
    #[inline]
    pub fn data_raw_folder() -> PathBuf {
        env::current_dir().unwrap().join(DATA_RAW_FOLDER)
    }
    #[inline]
    pub fn data_folder() -> PathBuf {
        env::current_dir().unwrap().join(DATA_FOLDER)
    }
    #[inline]
    pub fn platform_data_folder() -> PathBuf {
        env::current_dir()
            .unwrap()
            .join(DATA_FOLDER)
            .join(PC_FOLDER)
    }
}
