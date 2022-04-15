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
