pub struct Library {}

impl Library {
    #[inline]
    pub fn load(_filename: &str) -> Self {
        panic!("Trying to load a library from a web platform");
    }
    #[inline]
    pub fn get<T>(&self, _symbol: &str) -> Option<T> {
        None
    }

    #[inline]
    pub fn close(&mut self) {}
}
