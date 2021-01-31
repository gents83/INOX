use std::{env::consts::*, ffi::{OsStr, OsString}};

use super::platform_impl::platform::library as platform;

pub fn library_filename<S: AsRef<OsStr>>(name: S) -> OsString {
    let name = name.as_ref();
    let mut string = OsString::with_capacity(name.len() + DLL_PREFIX.len() + DLL_SUFFIX.len());
    string.push(DLL_PREFIX);
    string.push(name);
    string.push(DLL_SUFFIX);
    string
}


pub struct Library(platform::Library);

impl Library {
    pub fn new<S: AsRef<::std::ffi::OsStr>>(filename: S) -> Library{
        let _lib = platform::Library::load(filename);
        Library(_lib)
    }

    pub fn get<T>(&self, symbol: &str) -> T {
        unsafe {
            self.0.get(symbol)
        }
    }

    pub fn close(&mut self) {
        self.0.close();
    }
}
