use std::ffi::OsStr;

pub struct Library;

impl Library {
    #[inline]
    pub fn load<S: AsRef<OsStr>>(_filename: S) -> Library {
        Library
    }

    #[inline]
    pub fn get<T>(&self, _symbol: &str) -> Option<T> {
        None
    }

    #[inline]
    pub fn close(&mut self) {}
}
