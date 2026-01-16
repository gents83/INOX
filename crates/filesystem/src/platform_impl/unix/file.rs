use std::io::{BufReader, BufWriter, Read, Write};

use crate::File;

impl File {
    pub fn exists(&self) -> bool {
        if self.path.exists() && self.path.is_file() {
            return true;
        }
        false
    }

    pub fn load<F>(&mut self, f: F)
    where
        F: FnMut(&mut Vec<u8>) + 'static,
    {
        if self.exists() {
            let file = std::fs::File::open(self.path.as_path()).unwrap();
            let mut reader = BufReader::new(file);
            let mut bytes = self.bytes.write().unwrap();
            let bytes = bytes.as_mut();
            reader.read_to_end(bytes).ok();
        }
        self.apply(f);
    }

    pub fn save<F>(&mut self, f: F)
    where
        F: FnMut(&mut Vec<u8>) + 'static,
    {
        self.apply(f);
        {
            std::fs::create_dir_all(self.path.parent().unwrap()).ok();
            let file = std::fs::File::create(self.path.as_path()).unwrap();
            let mut writer = BufWriter::new(file);
            let bytes = self.bytes.read().unwrap();
            writer.write_all(bytes.as_slice()).ok();
        }
    }
}
