
use crate::File;

impl File {
    pub fn exists(&self) -> bool {
        true
    }

    pub fn load<F>(&mut self, mut _f: F)
    where
        F: FnMut(&mut Vec<u8>) + 'static, {
            eprintln!("Load not implemented for this platform");            
    }

    pub fn save<F>(&mut self, _f: F)
    where
        F: FnMut(&mut Vec<u8>) + 'static,
    {
        eprintln!("Save not implemented for this platform");
    }
}
