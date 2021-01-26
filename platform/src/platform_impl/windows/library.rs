use super::externs::*;
use super::types::*;

use std::env::consts::*;
use std::ffi::*;
use std::str;
use std::os::windows::ffi::*;


pub fn library_filename<S: AsRef<OsStr>>(name: S) -> OsString {
    let name = name.as_ref();
    let mut string = OsString::with_capacity(name.len() + DLL_PREFIX.len() + DLL_SUFFIX.len());
    string.push(DLL_PREFIX);
    string.push(name);
    string.push(DLL_SUFFIX);
    string
}


#[derive(Clone, Copy)]
pub struct Library(HMODULE);

impl Library
{
    pub fn load<S: AsRef<OsStr>>(filename: S) -> Library {
        let wide_filename: Vec<u16> = filename.as_ref().encode_wide().chain(Some(0)).collect();
        let handle = unsafe {
            LoadLibraryExW(wide_filename.as_ptr(), std::ptr::null_mut(), 0)
        };
        if handle.is_null() {
            panic!("Unable to load library {}", filename.as_ref().to_str().unwrap())
        }
        else {
            Library(handle)
        }
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

    pub fn close(self) {
        let result = unsafe { FreeLibrary(self.0) };
        if result == 0 {
            panic!("Unable to close library with error {}", result)
        } 
    }
}