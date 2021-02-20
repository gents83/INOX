use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

pub fn serialize<T>(data: &T, filepath: PathBuf)
where
    T: Serialize,
{
    let file = File::create(filepath).unwrap();
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &data).unwrap();
}

pub fn deserialize<'a, T>(data: &'a mut T, filepath: PathBuf)
where
    T: for<'de> Deserialize<'de>,
{
    if filepath.exists() {
        let file = File::open(filepath).unwrap();
        let reader = BufReader::new(file);
        if let Ok(result) = serde_json::from_reader(reader) {
            *data = result;
        }
    }
}
