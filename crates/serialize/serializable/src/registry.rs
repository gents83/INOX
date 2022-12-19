#![allow(improper_ctypes_definitions)]

use std::{
    any::{type_name, Any, TypeId},
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock},
};

pub static mut SERIALIZABLE_REGISTRY: Option<SerializableRegistryRc> = None;

pub fn check_serializable_registry(registry: &SerializableRegistryRc) {
    unsafe {
        if SERIALIZABLE_REGISTRY.is_none() {
            SERIALIZABLE_REGISTRY.replace(registry.clone());
        }
    }
}

pub trait TraitRegistry: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub enum DeserializeType {
    None,
    Adjacent {
        trait_object: &'static str,
        fields: &'static [&'static str],
    },
    Internal {
        trait_object: &'static str,
        tag: &'static str,
    },
    External {
        trait_object: &'static str,
    },
}

pub type SerializableRegistryRc = Arc<RwLock<SerializableRegistry>>;
pub type DeserializeFn<T> = fn(&mut dyn erased_serde::Deserializer) -> erased_serde::Result<Box<T>>;

pub struct Registry<T>
where
    T: ?Sized + 'static,
{
    pub map: BTreeMap<&'static str, Option<DeserializeFn<T>>>,
    pub names: Vec<&'static str>,
}
unsafe impl<T> Send for Registry<T> where T: ?Sized + 'static {}
unsafe impl<T> Sync for Registry<T> where T: ?Sized + 'static {}

impl<T> Registry<T>
where
    T: ?Sized + 'static,
{
    pub fn register_type(&mut self, name: &'static str, func: DeserializeFn<T>) {
        self.map.insert(name, Some(func));
        self.names.push(name);
        self.names.sort_unstable();
    }
    pub fn unregister_type(&mut self, name: &'static str) -> bool {
        self.map.remove(name);
        self.names.retain(|&x| x != name);
        self.map.is_empty()
    }
}

impl<T> Default for Registry<T>
where
    T: ?Sized + 'static,
{
    fn default() -> Self {
        Self {
            map: BTreeMap::new(),
            names: Vec::new(),
        }
    }
}

impl<T> TraitRegistry for Registry<T>
where
    T: ?Sized + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct SerializableRegistry {
    pub type_map: RwLock<HashMap<TypeId, Box<dyn TraitRegistry>>>,
}
unsafe impl Send for SerializableRegistry {}
unsafe impl Sync for SerializableRegistry {}

impl Drop for SerializableRegistry {
    fn drop(&mut self) {
        self.type_map.write().unwrap().clear();
    }
}

impl SerializableRegistry {
    pub fn register_type<T>(&self, name: &'static str, func: DeserializeFn<T>)
    where
        T: ?Sized + 'static,
    {
        let mut rwlock = self.type_map.write().unwrap();
        let registry = rwlock
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::<Registry<T>>::default());
        let registry = registry.as_any_mut().downcast_mut::<Registry<T>>().unwrap();

        registry.register_type(name, func);
    }
    pub fn unregister_type<T>(&self, name: &'static str)
    where
        T: ?Sized + 'static,
    {
        let mut rwlock = self.type_map.write().unwrap();
        let registry = rwlock
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::<Registry<T>>::default());
        let registry = registry.as_any_mut().downcast_mut::<Registry<T>>().unwrap();

        if registry.unregister_type(name) {
            rwlock.remove(&TypeId::of::<T>());
        }
    }
    pub fn deserialize<'de, T, D>(
        &self,
        deserializer: D,
        deserialize_type: DeserializeType,
    ) -> Result<Box<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: ?Sized + 'static,
    {
        use serde::de::Error;

        let rlock = self.type_map.read().unwrap();
        let untyped_registry = rlock.get(&TypeId::of::<T>()).unwrap_or_else(|| {
            panic!("Unable to find type in registry: {:?}", type_name::<T>());
        });
        let registry = untyped_registry
            .as_any()
            .downcast_ref::<Registry<T>>()
            .unwrap();
        let result = match deserialize_type {
            DeserializeType::External { trait_object } => {
                crate::externally::deserialize(deserializer, trait_object, registry)
            }
            DeserializeType::Internal { trait_object, tag } => {
                crate::internally::deserialize(deserializer, trait_object, tag, registry)
            }
            DeserializeType::Adjacent {
                trait_object,
                fields,
            } => crate::adjacently::deserialize(deserializer, trait_object, fields, registry),
            _ => std::result::Result::Err(D::Error::custom(format!(
                "Type {} not registered in registry",
                std::any::type_name::<T>()
            ))),
        };
        result
    }
}
