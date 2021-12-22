use crate::{AsSerializable, FromSerializable, Serializable};
use downcast_rs::{impl_downcast, Downcast};
use serde::Deserialize;
use std::{
    any::{type_name, Any, TypeId},
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

#[derive(Default)]
pub struct SerializableRegistry {
    type_registrations: HashMap<TypeId, SerializableTypeInfo>,
    trait_registrations: HashMap<TypeId, Box<dyn TraitInfo>>,
    names: HashMap<String, TypeId>,
    full_names: HashMap<String, TypeId>,
    ambiguous_names: HashSet<String>,
}

pub type SerializableRegistryRw = Arc<RwLock<SerializableRegistry>>;

pub trait TypeInfo {
    fn type_info() -> SerializableTypeInfo;
}

impl SerializableRegistry {
    pub fn register_type<T>(&mut self)
    where
        T: TypeInfo + 'static + Sized,
    {
        self.add_type(T::type_info());
    }
    pub fn unregister_type<T>(&mut self)
    where
        T: TypeInfo + 'static + Sized,
    {
        self.remove_type(T::type_info());
    }
    pub fn register_trait<T>(&mut self)
    where
        T: 'static + ?Sized + Serializable + Any,
    {
        self.add_trait::<T>(Box::new(SerializableTraitInfo::<T>::default()));
    }
    pub fn unregister_trait<T>(&mut self)
    where
        T: 'static + ?Sized + Serializable + Any,
    {
        self.remove_trait::<T>(SerializableTraitInfo::<T>::default());
    }
    pub fn register_type_with_trait<Trait, Type>(&mut self)
    where
        Trait: 'static + ?Sized + Serializable + Any,
        Type: TypeInfo
            + 'static
            + Sized
            + Serializable
            + FromSerializable
            + Any
            + AsSerializable<Trait>,
    {
        self.add_type(Type::type_info());
        let trait_id = TypeId::of::<Trait>();
        let trait_info = self.trait_registrations.get_mut(&trait_id).unwrap();
        let trait_info = trait_info
            .as_mut()
            .as_any_mut()
            .downcast_mut::<SerializableTraitInfo<Trait>>()
            .unwrap();
        trait_info.data.insert(
            type_name::<Type>().to_string(),
            Box::new(|v, r| {
                let b = Box::new(Type::from_serializable(v, r).unwrap());
                b.into_type()
            }),
        );
    }

    pub fn create_value_from_trait<Trait>(&self, value: &dyn Serializable) -> Box<Trait>
    where
        Trait: 'static + ?Sized + Serializable + Any,
        Box<Trait>: Serializable + Sized,
    {
        let trait_id = TypeId::of::<Trait>();
        let trait_info = self.trait_registrations.get(&trait_id).unwrap();
        let trait_info = trait_info
            .as_ref()
            .as_any()
            .downcast_ref::<SerializableTraitInfo<Trait>>()
            .unwrap();
        trait_info.data.get(value.type_name().as_str()).unwrap()(value, self)
    }

    fn add_name(&mut self, type_id: TypeId, short_name: &str, fullname: &str) {
        if self.names.contains_key(short_name) || self.ambiguous_names.contains(short_name) {
            // name is ambiguous. fall back to long names for all ambiguous types
            self.names.remove(short_name);
            self.ambiguous_names.insert(short_name.to_string());
        } else {
            self.names.insert(short_name.to_string(), type_id);
        }
        self.full_names.insert(fullname.to_string(), type_id);
    }

    fn remove_name(&mut self, short_name: &str, fullname: &str) {
        if self.names.contains_key(short_name) {
            self.names.remove(short_name);
        }
        self.full_names.remove(fullname);
    }

    fn add_type(&mut self, registration: SerializableTypeInfo) {
        self.add_name(
            registration.type_id,
            registration.name(),
            registration.fullname(),
        );
        self.type_registrations
            .insert(registration.type_id, registration);
    }

    fn remove_type(&mut self, registration: SerializableTypeInfo) {
        self.remove_name(registration.name(), registration.fullname());
        self.type_registrations.remove(&registration.type_id);
    }

    fn add_trait<T>(&mut self, registration: Box<SerializableTraitInfo<T>>)
    where
        T: 'static + ?Sized + Serializable + Any,
    {
        let typeid = registration.type_id;
        println!(
            "Trait {:?} id {:?} - reg_typeid = {:?}",
            registration.fullname(),
            typeid,
            registration.type_id()
        );
        self.add_name(
            registration.type_id,
            registration.name(),
            registration.fullname(),
        );
        self.trait_registrations
            .insert(registration.type_id, registration);
        let trait_info = self.trait_registrations.get(&typeid).unwrap();
        println!("reg_typeid = {:?}", trait_info.type_id());
    }

    fn remove_trait<T>(&mut self, registration: SerializableTraitInfo<T>)
    where
        T: 'static + ?Sized + Serializable + Any,
    {
        self.remove_name(registration.name(), registration.fullname());
        self.trait_registrations.remove(&registration.type_id);
    }

    pub fn get(&self, type_id: TypeId) -> Option<&SerializableTypeInfo> {
        self.type_registrations.get(&type_id)
    }

    pub fn get_mut(&mut self, type_id: TypeId) -> Option<&mut SerializableTypeInfo> {
        self.type_registrations.get_mut(&type_id)
    }

    pub fn get_with_fullname(&self, type_name: &str) -> Option<&SerializableTypeInfo> {
        self.full_names.get(type_name).and_then(|id| self.get(*id))
    }

    pub fn get_with_name_mut(&mut self, type_name: &str) -> Option<&mut SerializableTypeInfo> {
        self.full_names
            .get(type_name)
            .cloned()
            .and_then(move |id| self.get_mut(id))
    }

    pub fn get_with_name(&self, short_type_name: &str) -> Option<&SerializableTypeInfo> {
        self.names
            .get(short_type_name)
            .and_then(|id| self.type_registrations.get(id))
    }

    pub fn get_type_data<T: TypeData>(&self, type_id: TypeId) -> Option<&T> {
        self.get(type_id)
            .and_then(|registration| registration.data::<T>())
    }

    pub fn iter(&self) -> impl Iterator<Item = &SerializableTypeInfo> {
        self.type_registrations.values()
    }
}

type FromSerializableFn<Trait> = dyn Fn(&dyn Serializable, &SerializableRegistry) -> Box<Trait>;
trait TraitInfo: Any + Downcast {}
impl_downcast!(TraitInfo);
pub struct SerializableTraitInfo<Trait>
where
    Trait: 'static + ?Sized + Serializable + Any,
{
    type_id: TypeId,
    fullname: &'static str,
    data: HashMap<String, Box<FromSerializableFn<Trait>>>,
}
impl<T> TraitInfo for SerializableTraitInfo<T> where T: 'static + ?Sized + Serializable + Any {}

impl<T> Default for SerializableTraitInfo<T>
where
    T: 'static + ?Sized + Serializable + Any,
{
    fn default() -> Self {
        SerializableTraitInfo {
            type_id: TypeId::of::<T>(),
            fullname: type_name::<T>(),
            data: HashMap::new(),
        }
    }
}

impl<T> SerializableTraitInfo<T>
where
    T: 'static + ?Sized + Serializable + Any,
{
    pub fn name(&self) -> &str {
        self.fullname
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
    }

    pub fn fullname(&self) -> &'static str {
        self.fullname
    }
}

pub struct SerializableTypeInfo {
    type_id: TypeId,
    fullname: &'static str,
    data: HashMap<TypeId, Box<dyn TypeData>>,
}

impl SerializableTypeInfo {
    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn data<T>(&self) -> Option<&T>
    where
        T: TypeData,
    {
        self.data
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref())
    }

    pub fn data_mut<T>(&mut self) -> Option<&mut T>
    where
        T: TypeData,
    {
        self.data
            .get_mut(&TypeId::of::<T>())
            .and_then(|value| value.downcast_mut())
    }

    pub fn insert_with_type_id<T>(&mut self, type_id: TypeId, data: T)
    where
        T: TypeData,
    {
        self.data.insert(type_id, Box::new(data));
    }

    pub fn insert<T>(&mut self, data: T)
    where
        T: TypeData,
    {
        self.data.insert(TypeId::of::<T>(), Box::new(data));
    }

    pub fn of<T>() -> Self
    where
        T: Serializable + Sized,
    {
        let ty = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>();
        Self {
            type_id: ty,
            fullname: type_name,
            data: HashMap::default(),
        }
    }

    pub fn name(&self) -> &str {
        self.fullname
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
    }

    pub fn fullname(&self) -> &'static str {
        self.fullname
    }
}

impl Clone for SerializableTypeInfo {
    fn clone(&self) -> Self {
        let mut data = HashMap::default();
        for (id, type_data) in self.data.iter() {
            data.insert(*id, (*type_data).clone_type_data());
        }

        SerializableTypeInfo {
            type_id: self.type_id,
            fullname: self.fullname,
            data,
        }
    }
}

pub trait TypeData: Downcast {
    fn clone_type_data(&self) -> Box<dyn TypeData>;
}
impl_downcast!(TypeData);

impl<T> TypeData for T
where
    T: Clone + ?Sized + 'static,
{
    fn clone_type_data(&self) -> Box<dyn TypeData> {
        Box::new(self.clone())
    }
}

pub trait SerializableType<T> {
    fn from_value(&self, value: &dyn Serializable, registry: &SerializableRegistry) -> T;
    fn from_type_to_serializable() -> Self;
}

#[derive(Clone)]
pub struct SerializableDeserialize {
    #[allow(clippy::type_complexity)]
    pub func: fn(
        deserializer: &mut dyn erased_serde::Deserializer,
    ) -> Result<Box<dyn Serializable>, erased_serde::Error>,
}

impl SerializableDeserialize {
    pub fn deserialize<'de, D>(&self, deserializer: D) -> Result<Box<dyn Serializable>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut erased = <dyn erased_serde::Deserializer>::erase(deserializer);
        (self.func)(&mut erased)
            .map_err(<<D as serde::Deserializer<'de>>::Error as serde::de::Error>::custom)
    }
}

impl<T> SerializableType<T> for SerializableDeserialize
where
    T: for<'a> Deserialize<'a> + Serializable + FromSerializable,
{
    fn from_value(&self, value: &dyn Serializable, registry: &SerializableRegistry) -> T {
        T::from_serializable(value, registry).unwrap()
    }
    fn from_type_to_serializable() -> Self {
        SerializableDeserialize {
            func: |deserializer| Ok(Box::new(T::deserialize(deserializer)?)),
        }
    }
}
