use std::{
    env::{self, consts::*},
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use super::platform_impl::platform::library as platform;

pub const EXE_PATH: &str = "EXE_PATH";

#[inline]
pub fn library_filename<S: AsRef<OsStr>>(name: S) -> OsString {
    let name = name.as_ref();
    let mut string = OsString::with_capacity(name.len() + DLL_PREFIX.len() + DLL_SUFFIX.len());
    string.push(DLL_PREFIX);
    string.push(name);
    string.push(DLL_SUFFIX);
    string
}

#[inline]
pub fn compute_folder_and_filename(lib_path: &Path) -> (PathBuf, PathBuf) {
    let mut path = lib_path.to_path_buf();
    let mut filename = path.clone();
    if path.is_absolute() {
        filename = PathBuf::from(path.file_name().unwrap());
        path = PathBuf::from(path.parent().unwrap());
    } else if let Ok(current_exe_folder) = env::var("EXE_PATH") {
        path = PathBuf::from(current_exe_folder);
    } else {
        path = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    }
    path = path.canonicalize().unwrap();
    (path, filename)
}

pub struct Library(platform::Library);

impl Library {
    #[inline]
    pub fn new(filename: &str) -> Library {
        #[cfg(target_os = "ios")]
        return Library(platform::Library::load(filename));
        #[cfg(not(target_os = "ios"))]
        return Library(platform::Library::load(filename));
    }

    #[inline]
    pub fn get<T>(&self, symbol: &str) -> Option<T> {
        self.0.get(symbol)
    }

    #[inline]
    pub fn close(&mut self) {
        self.0.close();
    }
}
