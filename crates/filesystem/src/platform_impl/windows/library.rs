#![allow(
    bad_style,
    overflowing_literals,
    dead_code,
    improper_ctypes,
    improper_ctypes_definitions,
    clippy::upper_case_acronyms
)]

use std::str;
use std::{
    ffi::*,
    os::{
        raw::{c_char, c_int, c_ulong},
        windows::ffi::*,
    },
};

pub type LPCWSTR = *const u16;
pub type LPCSTR = *const c_char;
pub type BOOL = c_int;

#[derive(Clone, Copy)]
pub enum HINSTANCE__ {}
pub type HINSTANCE = *mut HINSTANCE__;

unsafe impl Send for HINSTANCE__ {}
unsafe impl Sync for HINSTANCE__ {}

pub enum __some_function {}
/// Pointer to a function with unknown type signature.
pub type FARPROC = *mut __some_function;

pub type HMODULE = HINSTANCE;
pub type HANDLE = *mut c_void;
pub type DWORD = c_ulong;

extern "system" {
    pub fn LoadLibraryExW(lpLibFileName: LPCWSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE;
    pub fn GetLastError() -> DWORD;
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> FARPROC;
    pub fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
}

pub struct Library(HMODULE);

impl Library {
    #[inline]
    pub fn load<S: AsRef<OsStr>>(filename: S) -> Library {
        let wide_filename: Vec<u16> = filename.as_ref().encode_wide().chain(Some(0)).collect();
        let handle = unsafe { LoadLibraryExW(wide_filename.as_ptr(), std::ptr::null_mut(), 0) };
        if handle.is_null() {
            let error = unsafe { GetLastError() };
            eprintln!(
                "Unable to load library {} and received error {}",
                filename.as_ref().to_str().unwrap(),
                error
            );
        }
        drop(wide_filename);
        Library(handle)
    }

    #[inline]
    pub fn get<T>(&self, symbol: &str) -> Option<T> {
        unsafe {
            let symbol_in_bytes = symbol.as_bytes();
            let mut name = Vec::with_capacity(symbol_in_bytes.len() + 1);
            name.resize(symbol_in_bytes.len(), 0);
            name.copy_from_slice(symbol_in_bytes);
            name.push(0);
            let ret = GetProcAddress(self.0, name.as_ptr() as *const i8);
            if ret.is_null() {
                return None;
            }
            Some(::std::mem::transmute_copy(&ret))
        }
    }

    #[inline]
    pub fn close(&mut self) {
        let mut res = unsafe { FreeLibrary(self.0) };
        while res > 0 {
            res = unsafe { FreeLibrary(self.0) };
        }
    }
}
impl Drop for Library {
    #[inline]
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.0) };
    }
}
