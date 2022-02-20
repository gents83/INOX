use inox_filesystem::File;
use inox_serializable::{check_serializable_registry, SerializableRegistryRc};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub trait SerializeFile {
    fn extension() -> &'static str;
    fn save_to_file(&self, path: &Path, registry: &SerializableRegistryRc)
    where
        Self: Serialize + Sized + Clone + 'static,
    {
        serialize_to_file(self, path, registry);
    }
    fn load_from_file(&mut self, path: &Path, registry: &SerializableRegistryRc)
    where
        Self: for<'de> Deserialize<'de> + Default,
    {
        *self = read_from_file(path, registry);
    }
}

#[inline]
pub fn serialize<T>(data: &T, registry: &SerializableRegistryRc) -> String
where
    T: Serialize,
{
    check_serializable_registry(registry);
    serde_json::to_string(&data).unwrap()
}

#[inline]
pub fn deserialize<'a, T>(
    serialized_data: &str,
    registry: &SerializableRegistryRc,
) -> Result<T, serde_json::Error>
where
    T: for<'de> Deserialize<'de>,
{
    check_serializable_registry(registry);
    serde_json::from_str(serialized_data)
}

#[inline]
pub fn serialize_to_file<T>(data: &T, filepath: &Path, registry: &SerializableRegistryRc)
where
    T: Serialize + Clone + Sized + SerializeFile + 'static,
{
    check_serializable_registry(registry);
    let data = data.clone();
    let mut file = File::new(filepath);
    file.apply(move |bytes| {
        serde_json::to_writer(bytes.as_mut_slice(), &data).unwrap();
    });
    file.save();
}

#[inline]
pub fn read_from_file<'a, T>(filepath: &Path, registry: &SerializableRegistryRc) -> T
where
    T: for<'de> Deserialize<'de> + Default + SerializeFile,
{
    let mut file = File::new(filepath);
    if file.exists() {
        check_serializable_registry(registry);
        file.load();
        match serde_json::from_reader(file.bytes().as_slice()) {
            Ok(data) => return data,
            Err(e) => {
                eprintln!(
                    "Error {} - Unable to deserialize file {}",
                    e,
                    filepath.to_str().unwrap_or("InvalidPath"),
                );
            }
        }
    } else {
        eprintln!(
            "Unable to find file {}",
            filepath.to_str().unwrap_or("InvalidPath"),
        );
    }
    T::default()
}
