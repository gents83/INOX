#![allow(dead_code)]

use std::ffi::{CString, OsStr};
use std::os::unix::ffi::OsStrExt;

pub struct Library(*mut std::ffi::c_void);

impl Library {
    pub fn load<S: AsRef<OsStr>>(filename: S) -> Library {
        let filename_cstr = CString::new(filename.as_ref().as_bytes()).unwrap();
        let handle = unsafe { libc::dlopen(filename_cstr.as_ptr(), libc::RTLD_LAZY | libc::RTLD_LOCAL) };
        if handle.is_null() {
            let error = unsafe { libc::dlerror() };
            if !error.is_null() {
                let error_str = unsafe { std::ffi::CStr::from_ptr(error) };
                eprintln!(
                    "Unable to load library {} and received error {:?}",
                    filename.as_ref().to_str().unwrap(),
                    error_str
                );
            }
        }
        Library(handle)
    }

    pub fn get<T>(&self, symbol: &str) -> Option<T> {
        let symbol_cstr = CString::new(symbol).unwrap();
        let symbol_ptr = unsafe { libc::dlsym(self.0, symbol_cstr.as_ptr()) };
        if symbol_ptr.is_null() {
            return None;
        }
        unsafe { Some(std::mem::transmute_copy(&symbol_ptr)) }
    }

    pub fn close(&mut self) {
        if !self.0.is_null() {
            unsafe { libc::dlclose(self.0) };
            self.0 = std::ptr::null_mut();
        }
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        self.close();
    }
}
