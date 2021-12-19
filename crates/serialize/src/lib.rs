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

use core::time::Duration;
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
    fn save_to_file(&self, _path: &Path)
    where
        Self: Serializable + Sized,
    {
        todo!();
        //serialize_to_file(self, path);
    }
    fn load_from_file(&mut self, _path: &Path)
    where
        Self: Serializable + Default,
    {
        todo!();
        //*self = read_from_file(path);
    }
}

#[inline]
pub fn read_from_file<T>(_filepath: &Path) -> T
where
    T: Serializable + Default + SerializeFile,
{
    todo!();
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
pub fn deserialize<T>(_serialized_data: &str) -> Result<T, serde_json::Error>
where
    T: Serializable,
{
    todo!();
}
