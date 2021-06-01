use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

#[inline]
pub fn serialize<T>(data: &T) -> String
where
    T: Serialize,
{
    serde_json::to_string(&data).unwrap()
}

#[inline]
pub fn deserialize<'a, T>(serialized_data: String) -> T
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(&serialized_data).unwrap()
}

#[inline]
pub fn serialize_to_file<T>(data: &T, filepath: PathBuf)
where
    T: Serialize + ?Sized,
{
    let file = File::create(filepath).unwrap();
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &data).unwrap();
}

#[inline]
pub fn deserialize_from_file<'a, T>(data: &'a mut T, filepath: PathBuf)
where
    T: for<'de> Deserialize<'de>,
{
    if filepath.exists() && filepath.is_file() {
        let file = File::open(filepath.clone()).unwrap();
        let reader = BufReader::new(file);
        if let Ok(result) = serde_json::from_reader(reader) {
            *data = result;
        } else {
            eprintln!(
                "Unable to deserialize file {}",
                filepath.to_str().unwrap_or("InvalidPath")
            );
        }
    }
}
