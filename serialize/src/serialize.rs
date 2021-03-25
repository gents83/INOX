use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

pub fn serialize<T>(data: &T) -> String
where
    T: Serialize,
{
    serde_json::to_string(&data).unwrap()
}

pub fn deserialize<'a, T>(serialized_data: String) -> T
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(&serialized_data).unwrap()
}

pub fn serialize_to_file<T>(data: &T, filepath: PathBuf)
where
    T: Serialize,
{
    let file = File::create(filepath).unwrap();
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &data).unwrap();
}

pub fn deserialize_from_file<'a, T>(data: &'a mut T, filepath: PathBuf)
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
