use super::externs::*;
use super::types::*;
use super::symbol::*;

use std::env::consts::*;
use std::ffi::*;
use std::str;
use std::marker;
use std::os::windows::ffi::*;


pub fn library_filename<S: AsRef<OsStr>>(name: S) -> OsString {
    let name = name.as_ref();
    let mut string = OsString::with_capacity(name.len() + DLL_PREFIX.len() + DLL_SUFFIX.len());
    string.push(DLL_PREFIX);
    string.push(name);
    string.push(DLL_SUFFIX);
    string
}


pub struct LibLoader(HMODULE);

impl LibLoader
{
    pub fn load<S: AsRef<OsStr>>(filename: S) -> LibLoader {
        let wide_filename: Vec<u16> = filename.as_ref().encode_wide().chain(Some(0)).collect();
        let handle = unsafe {
            LoadLibraryExW(wide_filename.as_ptr(), std::ptr::null_mut(), 0)
        };
        if handle.is_null() {
            panic!("Unable to load library {}", filename.as_ref().to_str().unwrap())
        }
        else
        {
            LibLoader(handle)
        }
    }
    
    pub unsafe fn get<T>(&self, symbol: &str) -> Symbol<T>{
        let ret = GetProcAddress(self.0, CString::new(symbol).unwrap().as_ptr());
        if ret.is_null() {
            panic!("Unable to get required symbol {}", str::from_utf8(::std::mem::transmute(symbol)).unwrap())
        } 
        else {
            Symbol {
                pointer: ret,
                pd: marker::PhantomData
            }
        }
    }

    pub fn close(self) {
        let result = unsafe { FreeLibrary(self.0) };
        if result != 0 {
            panic!("Unable to close library with error {}", result)
        } 
    }
}