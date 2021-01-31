use super::externs::*;
use super::types::*;

use std::ffi::*;
use std::str;
use std::os::windows::ffi::*;

pub struct Library(HMODULE);

impl Library
{
    pub fn load<S: AsRef<OsStr>>(filename: S) -> Library {
        let wide_filename: Vec<u16> = filename.as_ref().encode_wide().chain(Some(0)).collect();
        let handle = unsafe {
            LoadLibraryExW(wide_filename.as_ptr(), std::ptr::null_mut(), 0)
        };
        if handle.is_null() {
            let error = unsafe { GetLastError() };
            eprintln!("Unable to load library {} and received error {}", filename.as_ref().to_str().unwrap(), error);
        }
        drop(wide_filename);
        Library(handle)
    }
    //# Safety 
    pub unsafe fn get<T>(&self, symbol: &str) -> T{
        let fn_name = CString::new(symbol).unwrap();
        let ret = GetProcAddress(self.0, fn_name.as_ptr());
        if ret.is_null() {
            let error = GetLastError();
            eprintln!("Unable to get required symbol {} with error {}", symbol, error);
        }        
        ::std::mem::transmute_copy(&ret)
    }

    pub fn close(&mut self) {
        let mut res = unsafe { FreeLibrary(self.0) };
        while res > 0 {
            res =  unsafe { FreeLibrary(self.0) };
        } 
    }
}
impl Drop for Library {
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.0) };
    }
}