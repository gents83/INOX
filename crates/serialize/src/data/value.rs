use crate::{
    apply_in_list, is_list_equal, is_map_equal, serialization::SerializableValue,
    AsSerializableArray, AsSerializableList, DynamicSerializableMap, FromSerializable,
    Serializable, SerializableArray, SerializableDeserialize, SerializableDynamicEnum,
    SerializableEnum, SerializableEnumVariant, SerializableEnumVariantMut, SerializableList,
    SerializableListIterator, SerializableMap, SerializableMapIterator, SerializableMut,
    SerializableRef, SerializableRegistry, SerializableType, SerializableTypeInfo,
    SerializableVariantInfo, SerializableVariantInfoIterator, TypeInfo, Uid,
};

use sabi_serialize_derive::impl_serializable_value;
use serde::{Deserialize, Serialize};
use std::{
    any::Any,
    borrow::Cow,
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    ops::Range,
    path::PathBuf,
    time::Duration,
};

impl_serializable_value!(bool(Hash, PartialEq, Serialize, Deserialize));
impl_serializable_value!(u8(Hash, PartialEq, Serialize, Deserialize));
impl_serializable_value!(u16(Hash, PartialEq, Serialize, Deserialize));
impl_serializable_value!(u32(Hash, PartialEq, Serialize, Deserialize));
impl_serializable_value!(u64(Hash, PartialEq, Serialize, Deserialize));
impl_serializable_value!(u128(Hash, PartialEq, Serialize, Deserialize));
impl_serializable_value!(usize(Hash, PartialEq, Serialize, Deserialize));
impl_serializable_value!(i8(Hash, PartialEq, Serialize, Deserialize));
impl_serializable_value!(i16(Hash, PartialEq, Serialize, Deserialize));
impl_serializable_value!(i32(Hash, PartialEq, Serialize, Deserialize));
impl_serializable_value!(i64(Hash, PartialEq, Serialize, Deserialize));
impl_serializable_value!(i128(Hash, PartialEq, Serialize, Deserialize));
impl_serializable_value!(isize(Hash, PartialEq, Serialize, Deserialize));
impl_serializable_value!(f32(Serialize, Deserialize));
impl_serializable_value!(f64(Serialize, Deserialize));
impl_serializable_value!(String(Hash, PartialEq, Serialize, Deserialize));
impl_serializable_value!(PathBuf(Hash, PartialEq, Serialize, Deserialize));
impl_serializable_value!(Uid(Hash, PartialEq, Serialize, Deserialize));
impl_serializable_value!(HashSet<T: Serialize + Hash + Eq + Clone + for<'de> Deserialize<'de> + Send + Sync + 'static>(Serialize, Deserialize));
impl_serializable_value!(Range<T: Serialize + Clone + for<'de> Deserialize<'de> + Send + Sync + 'static>(Serialize, Deserialize));
impl_serializable_value!(Duration);

impl<T> SerializableList for Vec<T>
where
    T: FromSerializable,
{
    fn get_at(&self, index: usize) -> Option<&dyn Serializable> {
        <[T]>::get(self, index).map(|value| value as &dyn Serializable)
    }

    fn get_at_mut(&mut self, index: usize) -> Option<&mut dyn Serializable> {
        <[T]>::get_mut(self, index).map(|value| value as &mut dyn Serializable)
    }

    fn count(&self) -> usize {
        <[T]>::len(self)
    }

    fn iter_serializable(&self) -> SerializableListIterator {
        SerializableListIterator {
            list: self,
            index: 0,
        }
    }

    fn add(&mut self, value: Box<dyn Serializable>, registry: &SerializableRegistry) {
        let value = value.take::<T>().unwrap_or_else(|value| {
            T::from_serializable(&*value, registry).unwrap_or_else(|| {
                panic!(
                    "Attempted to push invalid value of type {}.",
                    value.type_name()
                )
            })
        });
        Vec::push(self, value);
    }
}

impl<T> Serializable for Vec<T>
where
    T: FromSerializable,
{
    #[inline]
    fn type_name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }

    #[inline]
    fn as_serializable(&self) -> &dyn Serializable {
        self
    }

    #[inline]
    fn as_serializable_mut(&mut self) -> &mut dyn Serializable {
        self
    }

    #[inline]
    fn any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn set(&mut self, value: &dyn Serializable, registry: &SerializableRegistry) {
        apply_in_list(self, value, registry);
    }

    #[inline]
    fn serializable_ref(&self) -> SerializableRef {
        SerializableRef::List(self)
    }

    #[inline]
    fn serializable_mut(&mut self) -> SerializableMut {
        SerializableMut::List(self)
    }

    #[inline]
    fn duplicate(&self) -> Box<dyn Serializable> {
        Box::new(self.clone_as_dynamic())
    }

    #[inline]
    fn compute_hash(&self) -> Option<u64> {
        None
    }

    #[inline]
    fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
        Some(is_list_equal(self, value))
    }

    #[inline]
    fn serializable_value(&self) -> Option<SerializableValue> {
        Some(SerializableValue::Boxed(Box::new(AsSerializableList(self))))
    }
}

impl<T> TypeInfo for Vec<T>
where
    T: FromSerializable + for<'de> Deserialize<'de>,
{
    fn type_info() -> SerializableTypeInfo {
        let mut info = SerializableTypeInfo::of::<Vec<T>>();
        info.insert::<SerializableDeserialize>(
            SerializableType::<Vec<T>>::from_type_to_serializable(),
        );
        info
    }
}

impl<T> FromSerializable for Vec<T>
where
    T: FromSerializable,
{
    fn from_serializable(
        value: &dyn Serializable,
        registry: &SerializableRegistry,
    ) -> Option<Self> {
        if let SerializableRef::List(ref_list) = value.serializable_ref() {
            let mut new_list = Self::with_capacity(ref_list.count());
            for field in ref_list.iter_serializable() {
                new_list.push(T::from_serializable(field, registry)?);
            }
            Some(new_list)
        } else {
            None
        }
    }
}

impl<K, V> SerializableMap for HashMap<K, V>
where
    K: FromSerializable + Eq + Hash,
    V: FromSerializable,
{
    fn get_with_key(&self, key: &dyn Serializable) -> Option<&dyn Serializable> {
        key.downcast_ref::<K>()
            .and_then(|key| HashMap::get(self, key))
            .map(|value| value as &dyn Serializable)
    }

    fn get_with_key_mut(&mut self, key: &dyn Serializable) -> Option<&mut dyn Serializable> {
        key.downcast_ref::<K>()
            .and_then(move |key| HashMap::get_mut(self, key))
            .map(|value| value as &mut dyn Serializable)
    }

    fn get_at(&self, index: usize) -> Option<(&dyn Serializable, &dyn Serializable)> {
        self.iter()
            .nth(index)
            .map(|(key, value)| (key as &dyn Serializable, value as &dyn Serializable))
    }

    fn count(&self) -> usize {
        HashMap::len(self)
    }

    fn iter_serializable(&self) -> SerializableMapIterator {
        SerializableMapIterator {
            map: self,
            index: 0,
        }
    }

    fn clone_as_dynamic(&self) -> DynamicSerializableMap {
        let mut dynamic_map = DynamicSerializableMap::default();
        dynamic_map.set_name(self.type_name());
        for (k, v) in HashMap::iter(self) {
            dynamic_map.insert_boxed(k.duplicate(), v.duplicate());
        }
        dynamic_map
    }
}

impl<K, V> Serializable for HashMap<K, V>
where
    K: FromSerializable + Eq + Hash,
    V: FromSerializable,
{
    #[inline]
    fn type_name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }

    #[inline]
    fn as_serializable(&self) -> &dyn Serializable {
        self
    }

    #[inline]
    fn as_serializable_mut(&mut self) -> &mut dyn Serializable {
        self
    }

    #[inline]
    fn any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn set(&mut self, value: &dyn Serializable, registry: &SerializableRegistry) {
        if let SerializableRef::Map(map_value) = value.serializable_ref() {
            for (key, value) in map_value.iter_serializable() {
                if let Some(v) = self.get_with_key_mut(key) {
                    v.set(value, registry)
                } else {
                    let k = K::from_serializable(key, registry).unwrap_or_else(|| {
                        panic!(
                            "Attempted to insert invalid key of type {} instead of {}",
                            key.type_name(),
                            std::any::type_name::<K>(),
                        )
                    });
                    let v = V::from_serializable(value, registry).unwrap_or_else(|| {
                        panic!(
                            "Attempted to insert invalid value of type {} instead of {}",
                            value.type_name(),
                            std::any::type_name::<V>(),
                        )
                    });
                    self.insert(k, v);
                }
            }
        } else {
            panic!("Attempted to apply a non-map type to a map type.");
        }
    }

    #[inline]
    fn serializable_ref(&self) -> SerializableRef {
        SerializableRef::Map(self)
    }

    #[inline]
    fn serializable_mut(&mut self) -> SerializableMut {
        SerializableMut::Map(self)
    }

    #[inline]
    fn duplicate(&self) -> Box<dyn Serializable> {
        Box::new(self.clone_as_dynamic())
    }

    #[inline]
    fn compute_hash(&self) -> Option<u64> {
        None
    }

    #[inline]
    fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
        Some(is_map_equal(self, value))
    }

    #[inline]
    fn serializable_value(&self) -> Option<SerializableValue> {
        None
    }
}

impl<K, V> TypeInfo for HashMap<K, V>
where
    K: FromSerializable + Eq + Hash + for<'de> Deserialize<'de>,
    V: FromSerializable + for<'de> Deserialize<'de>,
{
    fn type_info() -> SerializableTypeInfo {
        let mut info = SerializableTypeInfo::of::<HashMap<K, V>>();
        info.insert::<SerializableDeserialize>(
            SerializableType::<HashMap<K, V>>::from_type_to_serializable(),
        );
        info
    }
}

impl<K, V> FromSerializable for HashMap<K, V>
where
    K: FromSerializable + Eq + Hash,
    V: FromSerializable,
{
    fn from_serializable(
        value: &dyn Serializable,
        registry: &SerializableRegistry,
    ) -> Option<Self> {
        if let SerializableRef::Map(ref_map) = value.serializable_ref() {
            let mut new_map = Self::with_capacity(ref_map.count());
            for (key, value) in ref_map.iter_serializable() {
                let new_key = K::from_serializable(key, registry)?;
                let new_value = V::from_serializable(value, registry)?;
                new_map.insert(new_key, new_value);
            }
            Some(new_map)
        } else {
            None
        }
    }
}

impl Serializable for Cow<'static, str> {
    fn type_name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }

    #[inline]
    fn as_serializable(&self) -> &dyn Serializable {
        self
    }

    #[inline]
    fn as_serializable_mut(&mut self) -> &mut dyn Serializable {
        self
    }

    #[inline]
    fn any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn set(&mut self, value: &dyn Serializable, _registry: &SerializableRegistry) {
        let value = value.any();
        if let Some(value) = value.downcast_ref::<Self>() {
            *self = value.clone();
        } else {
            panic!("Value is not a {}.", std::any::type_name::<Self>());
        }
    }

    #[inline]
    fn serializable_ref(&self) -> SerializableRef {
        SerializableRef::Value(self)
    }

    #[inline]
    fn serializable_mut(&mut self) -> SerializableMut {
        SerializableMut::Value(self)
    }

    #[inline]
    fn duplicate(&self) -> Box<dyn Serializable> {
        Box::new(self.clone())
    }

    #[inline]
    fn compute_hash(&self) -> Option<u64> {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        Hash::hash(&std::any::Any::type_id(self), &mut hasher);
        Hash::hash(self, &mut hasher);
        Some(hasher.finish())
    }

    #[inline]
    fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
        let value = value.any();
        if let Some(value) = value.downcast_ref::<Self>() {
            Some(std::cmp::PartialEq::eq(self, value))
        } else {
            Some(false)
        }
    }

    #[inline]
    fn serializable_value(&self) -> Option<SerializableValue> {
        Some(SerializableValue::Ref(self))
    }
}

impl TypeInfo for Cow<'static, str> {
    fn type_info() -> SerializableTypeInfo {
        let mut info = SerializableTypeInfo::of::<Cow<'static, str>>();
        info.insert::<SerializableDeserialize>(
            SerializableType::<Cow<'static, str>>::from_type_to_serializable(),
        );
        info
    }
}

impl FromSerializable for Cow<'static, str> {
    fn from_serializable(
        value: &dyn Serializable,
        _registry: &SerializableRegistry,
    ) -> Option<Self> {
        Some(value.any().downcast_ref::<Cow<'static, str>>()?.clone())
    }
}

impl<T, const N: usize> Serializable for [T; N]
where
    T: Serializable,
{
    #[inline]
    fn type_name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }

    #[inline]
    fn as_serializable(&self) -> &dyn Serializable {
        self
    }

    #[inline]
    fn as_serializable_mut(&mut self) -> &mut dyn Serializable {
        self
    }

    #[inline]
    fn any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }
    #[inline]
    fn set(&mut self, value: &dyn Serializable, registry: &SerializableRegistry) {
        crate::apply_in_array(self, value, registry);
    }

    #[inline]
    fn serializable_ref(&self) -> SerializableRef {
        SerializableRef::Array(self)
    }

    #[inline]
    fn serializable_mut(&mut self) -> SerializableMut {
        SerializableMut::Array(self)
    }

    #[inline]
    fn duplicate(&self) -> Box<dyn Serializable> {
        Box::new(self.clone_as_dynamic())
    }

    #[inline]
    fn compute_hash(&self) -> Option<u64> {
        crate::array_hash(self)
    }

    #[inline]
    fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
        Some(crate::is_array_equal(self, value))
    }

    #[inline]
    fn serializable_value(&self) -> Option<SerializableValue> {
        Some(SerializableValue::Boxed(Box::new(AsSerializableArray(
            self,
        ))))
    }
}

impl<T, const N: usize> FromSerializable for [T; N]
where
    T: FromSerializable + Default,
{
    fn from_serializable(
        value: &dyn Serializable,
        registry: &SerializableRegistry,
    ) -> Option<Self> {
        if let SerializableRef::Array(ref_array) = value.serializable_ref() {
            let mut array = Vec::new();
            ref_array.iter_serializable().for_each(|value| {
                array.push(T::from_serializable(value, registry).unwrap());
            });
            Some(array.try_into().unwrap_or_else(|v: Vec<T>| {
                panic!("Expected a Vec of length {} but it was {}", N, v.len())
            }))
        } else {
            None
        }
    }
}

impl<T> TypeInfo for Option<T>
where
    T: Serializable + Clone + Send + Sync + 'static,
{
    fn type_info() -> SerializableTypeInfo {
        SerializableTypeInfo::of::<Option<T>>()
    }
}

impl<T> SerializableEnum for Option<T>
where
    T: Serializable + Clone + Send + Sync + 'static,
{
    fn variant(&self) -> SerializableEnumVariant<'_> {
        match self {
            Option::Some(new_type) => {
                SerializableEnumVariant::NewType(new_type as &dyn Serializable)
            }
            Option::None => SerializableEnumVariant::Unit,
        }
    }

    fn variant_mut(&mut self) -> SerializableEnumVariantMut<'_> {
        match self {
            Option::Some(new_type) => {
                SerializableEnumVariantMut::NewType(new_type as &mut dyn Serializable)
            }
            Option::None => SerializableEnumVariantMut::Unit,
        }
    }

    fn variant_info(&self) -> SerializableVariantInfo<'_> {
        let index = match self {
            Option::Some(_) => 0usize,
            Option::None => 1usize,
        };
        SerializableVariantInfo {
            index,
            name: self.get_index_name(index).unwrap(),
        }
    }

    fn get_index_name(&self, index: usize) -> Option<&'_ str> {
        match index {
            0usize => Some("Option::Some"),
            1usize => Some("Option::None"),
            _ => None,
        }
    }

    fn get_index_from_name(&self, name: &str) -> Option<usize> {
        match name {
            "Option::Some" => Some(0usize),
            "Option::None" => Some(1usize),
            _ => None,
        }
    }

    fn iter_variants_info(&self) -> SerializableVariantInfoIterator<'_> {
        SerializableVariantInfoIterator::new(self)
    }

    fn clone_as_dynamic(&self) -> SerializableDynamicEnum {
        let mut dynamic_enum = SerializableDynamicEnum::default();
        dynamic_enum.set_name(self.type_name());
        dynamic_enum.insert("Option::Some", self.as_ref().unwrap().clone());
        dynamic_enum.insert("Option::None", 1);
        dynamic_enum
    }
}
impl<T> Serializable for Option<T>
where
    T: Serializable + Clone + Send + Sync + 'static,
{
    #[inline]
    fn type_name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }

    fn as_serializable(&self) -> &dyn Serializable {
        self
    }

    fn as_serializable_mut(&mut self) -> &mut dyn Serializable {
        self
    }

    #[inline]
    fn any(&self) -> &dyn std::any::Any {
        self
    }

    #[inline]
    fn any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    #[inline]
    fn duplicate(&self) -> Box<dyn Serializable> {
        Box::new(self.clone())
    }

    #[inline]
    fn set(&mut self, value: &dyn Serializable, _registry: &SerializableRegistry) {
        let value = value.any();
        if let Some(value) = value.downcast_ref::<Self>() {
            *self = value.clone();
        } else {
            {
                panic!("Enum is not {}.", &std::any::type_name::<Self>());
            };
        }
    }

    fn serializable_ref(&self) -> SerializableRef {
        SerializableRef::Enum(self)
    }

    fn serializable_mut(&mut self) -> SerializableMut {
        SerializableMut::Enum(self)
    }

    fn serializable_value(&self) -> Option<SerializableValue> {
        None
    }

    fn compute_hash(&self) -> Option<u64> {
        None
    }

    fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
        Some(crate::is_enum_equal(self, value))
    }
}
