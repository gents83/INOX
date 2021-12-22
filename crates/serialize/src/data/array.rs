use crate::{
    serialization::SerializableValue, Serializable, SerializableMut, SerializableRef,
    SerializableRegistry,
};
use serde::ser::SerializeSeq;
use std::any::Any;

pub trait SerializableArray: Serializable {
    fn get(&self, index: usize) -> Option<&dyn Serializable>;
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Serializable>;
    fn count(&self) -> usize;
    fn values(&self) -> Box<[Box<dyn Serializable>]>;
    fn is_empty(&self) -> bool {
        self.count() == 0
    }
    fn iter_serializable(&self) -> SerializableArrayIterator;
    fn clone_as_dynamic(&self) -> SerializableDynamicArray {
        SerializableDynamicArray {
            name: self.type_name(),
            values: self
                .iter_serializable()
                .map(|value| value.duplicate())
                .collect(),
        }
    }
}

impl<T, const N: usize> SerializableArray for [T; N]
where
    T: Serializable,
{
    fn values(&self) -> Box<[Box<dyn Serializable>]> {
        self.iter()
            .map(|value| value.duplicate())
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
    #[inline]
    fn get(&self, index: usize) -> Option<&dyn Serializable> {
        <[T]>::get(self, index).map(|value| value as &dyn Serializable)
    }

    #[inline]
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Serializable> {
        <[T]>::get_mut(self, index).map(|value| value as &mut dyn Serializable)
    }

    #[inline]
    fn count(&self) -> usize {
        N
    }

    #[inline]
    fn iter_serializable(&self) -> SerializableArrayIterator {
        SerializableArrayIterator {
            array: self,
            index: 0,
        }
    }
}

pub struct AsSerializableArray<'a>(pub &'a dyn SerializableArray);

impl<'a> serde::Serialize for AsSerializableArray<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::array_serialize(self.0, serializer)
    }
}

pub struct SerializableDynamicArray {
    name: String,
    values: Box<[Box<dyn Serializable>]>,
}

impl SerializableDynamicArray {
    #[inline]
    pub fn new(values: Box<[Box<dyn Serializable>]>) -> Self {
        Self {
            name: String::default(),
            values,
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

impl Serializable for SerializableDynamicArray {
    #[inline]
    fn type_name(&self) -> String {
        self.name.clone()
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
        apply_in_array(self, value, registry);
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
        array_hash(self)
    }

    #[inline]
    fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
        Some(is_array_equal(self, value))
    }

    #[inline]
    fn serializable_value(&self) -> Option<SerializableValue> {
        Some(SerializableValue::Ref(self))
    }
}

impl SerializableArray for SerializableDynamicArray {
    fn values(&self) -> Box<[Box<dyn Serializable>]> {
        self.values.iter().map(|value| value.duplicate()).collect()
    }
    #[inline]
    fn get(&self, index: usize) -> Option<&dyn Serializable> {
        self.values.get(index).map(|value| &**value)
    }

    #[inline]
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Serializable> {
        self.values.get_mut(index).map(|value| &mut **value)
    }
    #[inline]
    fn count(&self) -> usize {
        self.values.len()
    }

    #[inline]
    fn iter_serializable(&self) -> SerializableArrayIterator {
        SerializableArrayIterator {
            array: self,
            index: 0,
        }
    }

    #[inline]
    fn clone_as_dynamic(&self) -> SerializableDynamicArray {
        SerializableDynamicArray {
            name: self.name.clone(),
            values: self.values.iter().map(|value| value.duplicate()).collect(),
        }
    }
}

pub struct SerializableArrayIterator<'a> {
    pub(crate) array: &'a dyn SerializableArray,
    pub(crate) index: usize,
}

impl<'a> Iterator for SerializableArrayIterator<'a> {
    type Item = &'a dyn Serializable;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.array.get(self.index);
        self.index += 1;
        value
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.array.count();
        (size, Some(size))
    }
}

impl<'a> ExactSizeIterator for SerializableArrayIterator<'a> {}

impl<'a> serde::Serialize for dyn SerializableArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        array_serialize(self, serializer)
    }
}

impl serde::Serialize for SerializableDynamicArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        array_serialize(self, serializer)
    }
}

#[inline]
pub fn array_serialize<A, S>(array: &A, serializer: S) -> Result<S::Ok, S::Error>
where
    A: SerializableArray + ?Sized,
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(array.count()))?;
    for element in array.iter_serializable() {
        let serializable = element.serializable_value().ok_or_else(|| {
            serde::ser::Error::custom(format!(
                "Type '{}' does not support `Serializable` serialization",
                element.type_name()
            ))
        })?;
        seq.serialize_element(serializable.borrow())?;
    }
    seq.end()
}

#[inline]
pub fn array_hash<T>(array: &T) -> Option<u64>
where
    T: SerializableArray,
{
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    std::any::Any::type_id(array).hash(&mut hasher);
    array.count().hash(&mut hasher);
    for value in array.iter_serializable() {
        hasher.write_u64(value.compute_hash()?)
    }
    Some(hasher.finish())
}

#[inline]
pub fn apply_in_array<A>(array: &mut A, value: &dyn Serializable, registry: &SerializableRegistry)
where
    A: SerializableArray,
{
    if let SerializableRef::Array(serializable_array) = value.serializable_ref() {
        for (i, value) in serializable_array.iter_serializable().enumerate() {
            let v = array.get_mut(i).unwrap();
            v.set(value, registry);
        }
    } else {
        panic!("Attempted to apply a non-`Array` type to an `Array` type.");
    }
}

#[inline]
pub fn is_array_equal<A: SerializableArray>(array: &A, value: &dyn Serializable) -> bool {
    match value.serializable_ref() {
        SerializableRef::Array(serializable_array)
            if serializable_array.count() == array.count() =>
        {
            for (a, b) in array
                .iter_serializable()
                .zip(serializable_array.iter_serializable())
            {
                if let Some(false) | None = a.is_equal(b) {
                    return false;
                }
            }
        }
        _ => return false,
    }

    true
}
