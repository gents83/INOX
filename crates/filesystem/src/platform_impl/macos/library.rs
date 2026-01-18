#![cfg(target_os = "macos")]

use crate::library::Library;

pub fn open_lib(name: &str) -> Option<Library> {
    let filename = format!("lib{name}.dylib");
    if let Ok(lib) = unsafe { ::libloading::Library::new(filename) } {
        return Some(Library(lib));
    }
    None
}
