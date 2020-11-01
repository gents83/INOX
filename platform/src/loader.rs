use super::platform_impl::platform::loader as platform;
use super::symbol::*;

pub struct LibLoader(platform::LibLoader);

impl LibLoader {
    pub fn new<S: AsRef<::std::ffi::OsStr>>(filename: S) -> LibLoader{
        let _lib = platform::LibLoader::load(filename);
        LibLoader(_lib)
    }

    pub fn get<'lib, T>(&'lib self, symbol: &str) -> Symbol<'lib, T> {
        unsafe {
            let ret = self.0.get(symbol);
            Symbol::from_raw(ret, self)
        }
    }

    pub fn close(self) {
        self.0.close()
    }
}