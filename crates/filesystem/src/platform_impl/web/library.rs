pub struct Library {}
impl Library {
    #[inline]
    pub fn load(filename: &str) -> Self {
        Self {}
    }

    #[inline]
    pub fn get<T>(&self, symbol: &str) -> Option<T> {
        None
    }

    #[inline]
    pub fn close(&mut self) {}
}
