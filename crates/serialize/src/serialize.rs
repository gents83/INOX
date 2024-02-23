use inox_filesystem::File;
use inox_serializable::{check_serializable_registry, SerializableRegistryRc};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub enum SerializationType {
    Binary,
    Json,
}

pub trait SerializeFile {
    fn extension() -> &'static str;
    fn save_to_file(
        &self,
        path: &Path,
        registry: SerializableRegistryRc,
        serialization_type: SerializationType,
    ) where
        Self: Serialize + Sized + 'static + Clone,
    {
        serialize_to_file(self, path, registry, serialization_type);
    }
    fn load_from_file(
        &mut self,
        path: &Path,
        registry: SerializableRegistryRc,
        serialization_type: SerializationType,

        f: Box<dyn FnMut(Self) + 'static>,
    ) where
        Self: for<'de> Deserialize<'de> + Default + 'static,
    {
        read_from_file(path, registry, serialization_type, f);
    }
}

#[inline]
pub fn serialize_to_text<T>(data: &T, registry: SerializableRegistryRc) -> Vec<u8>
where
    T: Serialize,
{
    check_serializable_registry(registry);
    serde_json::to_vec(&data).unwrap()
}

#[inline]
pub fn deserialize_from_text<'a, T>(
    serialized_data: &[u8],
    registry: SerializableRegistryRc,
) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    check_serializable_registry(registry);
    match serde_json::from_str(
        String::from_utf8(serialized_data.to_vec())
            .unwrap()
            .as_str(),
    ) {
        Ok(data) => Some(data),
        Err(e) => {
            eprintln!("Error {} - Unable to deserialize", e,);
            None
        }
    }
}

#[inline]
pub fn serialize<T>(data: &T, registry: SerializableRegistryRc) -> Vec<u8>
where
    T: Serialize,
{
    check_serializable_registry(registry);
    let mut s = flexbuffers::FlexbufferSerializer::new();
    data.serialize(&mut s).unwrap();
    s.view().to_vec()
}

#[inline]
pub fn deserialize<'a, T>(serialized_data: &[u8], registry: SerializableRegistryRc) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    check_serializable_registry(registry);
    match flexbuffers::Reader::get_root(serialized_data) {
        Ok(reader) => match T::deserialize(reader) {
            Ok(data) => {
                return Some(data);
            }
            Err(e) => {
                eprintln!("Error {} - Unable to deserialize", e,);
            }
        },
        Err(e) => {
            eprintln!("Error {} - Unable to deserialize", e,);
        }
    };
    None
}

#[inline]
pub fn serialize_to_file<T>(
    data: &T,
    filepath: &Path,
    registry: SerializableRegistryRc,
    serialization_type: SerializationType,
) where
    T: Serialize + Sized + SerializeFile + 'static + Clone,
{
    let data = data.clone();
    let mut file = File::new(filepath);
    file.save(move |bytes| {
        let b = match serialization_type {
            SerializationType::Binary => serialize(&data, registry.clone()),
            SerializationType::Json => serialize_to_text(&data, registry.clone()),
        };
        bytes.extend_from_slice(&b);
    });
}

#[inline]
pub fn read_from_file<'a, T>(
    filepath: &Path,
    registry: SerializableRegistryRc,
    serialization_type: SerializationType,
    mut f: Box<dyn FnMut(T) + 'static>,
) -> bool
where
    T: for<'de> Deserialize<'de> + SerializeFile + 'static,
{
    let mut file = File::new(filepath);
    if file.exists() {
        check_serializable_registry(registry.clone());
        let path = filepath.to_path_buf();
        file.load(move |bytes| {
            if let Some(data) = match serialization_type {
                SerializationType::Binary => deserialize(bytes, registry.clone()),
                SerializationType::Json => deserialize_from_text(bytes, registry.clone()),
            } {
                f(data);
            } else {
                eprintln!(
                    "Unable to deserialize file {}",
                    path.to_str().unwrap_or("InvalidPath"),
                );
            }
        });
        return true;
    }
    eprintln!(
        "Unable to find file {}",
        filepath.to_str().unwrap_or("InvalidPath"),
    );
    false
}
