use serde::ser::SerializeSeq;
use std::any::Any;

use crate::{
    serialization::SerializableValue, Serializable, SerializableMut, SerializableRef,
    SerializableRegistry,
};

/// An ordered, mutable list of [Reflect] items. This corresponds to types like [std::vec::Vec].
pub trait SerializableList: Serializable {
    fn get_at(&self, index: usize) -> Option<&dyn Serializable>;
    fn get_at_mut(&mut self, index: usize) -> Option<&mut dyn Serializable>;
    fn add(&mut self, value: Box<dyn Serializable>, registry: &SerializableRegistry);
    fn count(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.count() == 0
    }
    fn iter_serializable(&self) -> SerializableListIterator;
    fn clone_as_dynamic(&self) -> SerializableDynamicList {
        SerializableDynamicList {
            name: self.type_name(),
            values: self
                .iter_serializable()
                .map(|value| value.duplicate())
                .collect(),
        }
    }
}

pub struct AsSerializableList<'a>(pub &'a dyn SerializableList);

impl<'a> serde::Serialize for AsSerializableList<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::list_serialize(self.0, serializer)
    }
}

#[derive(Default)]
pub struct SerializableDynamicList {
    name: String,
    values: Vec<Box<dyn Serializable>>,
}

impl SerializableDynamicList {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn push<T: Serializable>(&mut self, value: T) {
        self.values.push(Box::new(value));
    }

    pub fn push_boxed(&mut self, value: Box<dyn Serializable>) {
        self.values.push(value);
    }
}

impl SerializableList for SerializableDynamicList {
    fn get_at(&self, index: usize) -> Option<&dyn Serializable> {
        self.values.get(index).map(|value| value.as_ref())
    }

    fn get_at_mut(&mut self, index: usize) -> Option<&mut dyn Serializable> {
        self.values.get_mut(index).map(|value| value.as_mut())
    }

    fn count(&self) -> usize {
        self.values.len()
    }

    fn clone_as_dynamic(&self) -> SerializableDynamicList {
        SerializableDynamicList {
            name: self.name.clone(),
            values: self.values.iter().map(|value| value.duplicate()).collect(),
        }
    }

    fn iter_serializable(&self) -> SerializableListIterator {
        SerializableListIterator {
            list: self,
            index: 0,
        }
    }

    fn add(&mut self, value: Box<dyn Serializable>, _registry: &SerializableRegistry) {
        SerializableDynamicList::push_boxed(self, value);
    }
}

impl Serializable for SerializableDynamicList {
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
        list_hash(self)
    }

    #[inline]
    fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
        Some(is_list_equal(self, value))
    }

    #[inline]
    fn serializable_value(&self) -> Option<SerializableValue> {
        None
    }
}

pub struct SerializableListIterator<'a> {
    pub(crate) list: &'a dyn SerializableList,
    pub(crate) index: usize,
}

impl<'a> Iterator for SerializableListIterator<'a> {
    type Item = &'a dyn Serializable;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.list.get_at(self.index);
        self.index += 1;
        value
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.list.count();
        (size, Some(size))
    }
}

impl<'a> ExactSizeIterator for SerializableListIterator<'a> {}

impl<'a> serde::Serialize for dyn SerializableList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        list_serialize(self, serializer)
    }
}

impl serde::Serialize for SerializableDynamicList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        list_serialize(self, serializer)
    }
}

#[inline]
pub fn list_serialize<A, S>(list: &A, serializer: S) -> Result<S::Ok, S::Error>
where
    A: SerializableList + ?Sized,
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(list.count()))?;
    for element in list.iter_serializable() {
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
pub fn list_hash<T>(list: &T) -> Option<u64>
where
    T: SerializableList,
{
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    std::any::Any::type_id(list).hash(&mut hasher);
    list.count().hash(&mut hasher);
    for value in list.iter_serializable() {
        hasher.write_u64(value.compute_hash()?)
    }
    Some(hasher.finish())
}

#[inline]
pub fn apply_in_list<T>(a: &mut T, b: &dyn Serializable, registry: &SerializableRegistry)
where
    T: SerializableList,
{
    if let SerializableRef::List(list_value) = b.serializable_ref() {
        for (i, value) in list_value.iter_serializable().enumerate() {
            if i < a.count() {
                if let Some(v) = SerializableList::get_at_mut(a, i) {
                    v.set(value, registry);
                }
            } else {
                a.add(value.duplicate(), registry);
            }
        }
    } else {
        panic!("Attempted to apply a non-list type to a list type.");
    }
}

#[inline]
pub fn is_list_equal<T>(a: &T, b: &dyn Serializable) -> bool
where
    T: SerializableList,
{
    let list = if let SerializableRef::List(list) = b.serializable_ref() {
        list
    } else {
        return false;
    };

    if a.count() != list.count() {
        return false;
    }

    for (a_value, b_value) in a.iter_serializable().zip(list.iter_serializable()) {
        if let Some(false) | None = a_value.is_equal(b_value) {
            return false;
        }
    }

    true
}
