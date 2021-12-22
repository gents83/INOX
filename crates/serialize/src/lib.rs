pub mod data;
pub mod serializable;
pub mod serializable_registry;
pub mod serialization;
mod test;
pub mod uuid;

pub use self::data::*;
pub use self::serializable::*;
pub use self::serializable_registry::*;
pub use self::serialization::*;
pub use self::uuid::*;

pub use erased_serde;
pub use sabi_serialize_derive::*;
pub use serde;
use serde::de::DeserializeSeed;
use serde_json::de::StrRead;

use core::time::Duration;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::path::PathBuf;

pub fn register_common_types(serializable_registry: &mut SerializableRegistry) {
    serializable_registry.register_type::<bool>();
    serializable_registry.register_type::<u8>();
    serializable_registry.register_type::<u16>();
    serializable_registry.register_type::<u32>();
    serializable_registry.register_type::<u64>();
    serializable_registry.register_type::<u128>();
    serializable_registry.register_type::<usize>();
    serializable_registry.register_type::<i8>();
    serializable_registry.register_type::<i16>();
    serializable_registry.register_type::<i32>();
    serializable_registry.register_type::<i64>();
    serializable_registry.register_type::<i128>();
    serializable_registry.register_type::<isize>();
    serializable_registry.register_type::<f32>();
    serializable_registry.register_type::<f64>();
    serializable_registry.register_type::<String>();
    serializable_registry.register_type::<PathBuf>();
    serializable_registry.register_type::<Uid>();
    serializable_registry.register_type::<Duration>();
}

pub fn unregister_common_types(serializable_registry: &mut SerializableRegistry) {
    serializable_registry.unregister_type::<bool>();
    serializable_registry.unregister_type::<u8>();
    serializable_registry.unregister_type::<u16>();
    serializable_registry.unregister_type::<u32>();
    serializable_registry.unregister_type::<u64>();
    serializable_registry.unregister_type::<u128>();
    serializable_registry.unregister_type::<usize>();
    serializable_registry.unregister_type::<i8>();
    serializable_registry.unregister_type::<i16>();
    serializable_registry.unregister_type::<i32>();
    serializable_registry.unregister_type::<i64>();
    serializable_registry.unregister_type::<i128>();
    serializable_registry.unregister_type::<isize>();
    serializable_registry.unregister_type::<f32>();
    serializable_registry.unregister_type::<f64>();
    serializable_registry.unregister_type::<String>();
    serializable_registry.unregister_type::<PathBuf>();
    serializable_registry.unregister_type::<Uid>();
    serializable_registry.unregister_type::<Duration>();
}

pub trait SerializeFile {
    fn extension() -> &'static str;
    fn save_to_file(&self, path: &Path, serializable_registry: &SerializableRegistry)
    where
        Self: Serializable + Sized,
    {
        write_to_file(self, path, serializable_registry);
    }
    fn load_from_file(&mut self, path: &Path, serializable_registry: &SerializableRegistry)
    where
        Self: Serializable + Default,
    {
        *self = read_from_file(path, serializable_registry);
    }
}

#[inline]
pub fn write_to_file<T>(data: &T, filepath: &Path, serializable_registry: &SerializableRegistry)
where
    T: Serializable + Sized + SerializeFile,
{
    let file = File::create(filepath).unwrap();
    let writer = BufWriter::new(file);
    let serializable_serializer = SerializableSerializer::new(data, serializable_registry);
    serde_json::to_writer(writer, &serializable_serializer).unwrap();
}

#[inline]
pub fn read_from_file<T>(filepath: &Path, serializable_registry: &SerializableRegistry) -> T
where
    T: Serializable + Default + SerializeFile,
{
    if filepath.exists() && filepath.is_file() {
        let file = File::open(filepath).unwrap();
        let reader = BufReader::new(file);

        let mut json_deserializer = serde_json::Deserializer::from_reader(reader);
        let deserializer = SerializableDeserializer::new(serializable_registry);

        let value = deserializer.deserialize(&mut json_deserializer).unwrap();
        let mut v = T::default();

        v.set_from(value.as_ref(), serializable_registry);
        return v;
    } else {
        eprintln!(
            "Unable to find file {}",
            filepath.to_str().unwrap_or("InvalidPath"),
        );
    }
    T::default()
}

#[inline]
pub fn serialize<T>(data: &T, serializable_registry: &SerializableRegistry) -> String
where
    T: Serializable,
{
    let serializable_serializer = SerializableSerializer::new(data, serializable_registry);
    serde_json::to_string_pretty(&serializable_serializer).unwrap()
}

#[inline]
pub fn deserialize<T>(
    serialized_data: &str,
    serializable_registry: &SerializableRegistry,
) -> Result<T, serde_json::Error>
where
    T: Serializable + Default,
{
    let mut json_deserializer = serde_json::Deserializer::new(StrRead::new(serialized_data));
    let deserializer = SerializableDeserializer::new(serializable_registry);

    let value = deserializer.deserialize(&mut json_deserializer).unwrap();
    let mut v = T::default();

    v.set_from(value.as_ref(), serializable_registry);
    Ok(v)
}
