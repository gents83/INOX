use super::externs::*;
use super::types::*;

use std::marker;
use std::mem;
use std::fmt;

pub struct Symbol<T> {
    pub pointer: FARPROC,
    pub pd: marker::PhantomData<T>
}

impl<T> Symbol<T> {
    pub fn into_raw(self) -> FARPROC {
        let pointer = self.pointer;
        mem::forget(self);
        pointer
    }
}

unsafe impl<T: Send> Send for Symbol<T> {}
unsafe impl<T: Sync> Sync for Symbol<T> {}

impl<T> Clone for Symbol<T> {
    fn clone(&self) -> Symbol<T> {
        Symbol { ..*self }
    }
}

impl<T> ::std::ops::Deref for Symbol<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            // Additional reference level for a dereference on `deref` return value.
            &*(&self.pointer as *const *mut _ as *const T)
        }
    }
}

impl<T> fmt::Debug for Symbol<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("Symbol@{:p}", self.pointer))
    }
}
