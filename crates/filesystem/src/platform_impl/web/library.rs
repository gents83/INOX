use crate::library::Library as LibraryWrapper;

pub struct Library {}
impl Library {
    #[inline]
    pub fn load(_filename: &str) -> Self {
        Self {}
    }

    #[inline]
    pub fn get<T>(&self, _symbol: &str) -> Option<T> {
        None
    }

    #[inline]
    pub fn close(&mut self) {}
}

pub fn open_lib(name: &str) -> Option<LibraryWrapper> {
    Some(LibraryWrapper(Library::load(name)))
}
