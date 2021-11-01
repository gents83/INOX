use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
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
pub fn serialize_to_file<T>(data: &T, filepath: &Path)
where
    T: Serialize + ?Sized,
{
    let file = File::create(filepath).unwrap();
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &data).unwrap();
}

#[inline]
pub fn read_from_file<'a, T>(filepath: &Path) -> T
where
    T: for<'de> Deserialize<'de> + Default,
{
    if filepath.exists() && filepath.is_file() {
        let file = File::open(filepath).unwrap();
        let reader = BufReader::new(file);
        if let Ok(result) = serde_json::from_reader(reader) {
            return result;
        } else {
            eprintln!(
                "Unable to deserialize file {}",
                filepath.to_str().unwrap_or("InvalidPath"),
            );
        }
    }
    T::default()
}

#[inline]
pub fn deserialize_from_file<'a, T>(data: &mut T, filepath: &Path)
where
    T: for<'de> Deserialize<'de>,
{
    if filepath.exists() && filepath.is_file() {
        let file = File::open(filepath).unwrap();
        let reader = BufReader::new(file);
        if let Ok(result) = serde_json::from_reader(reader) {
            *data = result;
        } else {
            eprintln!(
                "Unable to deserialize file {}",
                filepath.to_str().unwrap_or("InvalidPath"),
            );
        }
    }
}
