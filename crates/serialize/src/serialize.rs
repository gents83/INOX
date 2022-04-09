use inox_filesystem::File;
use inox_serializable::{check_serializable_registry, SerializableRegistryRc};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub trait SerializeFile {
    fn extension() -> &'static str;
    fn save_to_file(&self, path: &Path, registry: &SerializableRegistryRc)
    where
        Self: Serialize + Sized + 'static + Clone,
    {
        serialize_to_file(self, path, registry);
    }
    fn load_from_file(
        &mut self,
        path: &Path,
        registry: &SerializableRegistryRc,
        f: Box<dyn FnMut(Self) + 'static>,
    ) where
        Self: for<'de> Deserialize<'de> + Default + 'static,
    {
        read_from_file(path, registry, f);
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
    T: Serialize + Sized + SerializeFile + 'static + Clone,
{
    check_serializable_registry(registry);
    let data = data.clone();
    let mut file = File::new(filepath);
    file.save(move |bytes| {
        let string = serde_json::to_string(&data).unwrap();
        bytes.extend_from_slice(string.as_bytes());
    });
}

#[inline]
pub fn read_from_file<'a, T>(
    filepath: &Path,
    registry: &SerializableRegistryRc,
    mut f: Box<dyn FnMut(T) + 'static>,
) where
    T: for<'de> Deserialize<'de> + SerializeFile + 'static,
{
    let mut file = File::new(filepath);
    if file.exists() {
        check_serializable_registry(registry);
        let path = filepath.to_path_buf();
        file.load(
            move |bytes| match serde_json::from_reader(bytes.as_slice()) {
                Ok(data) => {
                    f(data);
                }
                Err(e) => {
                    eprintln!(
                        "Error {} - Unable to deserialize file {}",
                        e,
                        path.to_str().unwrap_or("InvalidPath"),
                    );
                }
            },
        );
    } else {
        eprintln!(
            "Unable to find file {}",
            filepath.to_str().unwrap_or("InvalidPath"),
        );
    }
}
