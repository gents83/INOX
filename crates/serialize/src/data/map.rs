use std::{
    any::Any,
    collections::{hash_map::Entry, HashMap},
};

use crate::{
    serialization::SerializableValue, Serializable, SerializableMut, SerializableRef,
    SerializableRegistry,
};

pub trait SerializableMap: Serializable {
    fn get_with_key(&self, key: &dyn Serializable) -> Option<&dyn Serializable>;
    fn get_with_key_mut(&mut self, key: &dyn Serializable) -> Option<&mut dyn Serializable>;
    fn get_at(&self, index: usize) -> Option<(&dyn Serializable, &dyn Serializable)>;
    fn count(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.count() == 0
    }
    fn iter_serializable(&self) -> SerializableMapIterator;
    fn clone_as_dynamic(&self) -> DynamicSerializableMap;
}

const HASH_ERROR_MESSAGE: &str = "This key seems to not support hashing";

#[derive(Default)]
pub struct DynamicSerializableMap {
    name: String,
    values: Vec<(Box<dyn Serializable>, Box<dyn Serializable>)>,
    indices: HashMap<u64, usize>,
}

impl DynamicSerializableMap {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn insert<K: Serializable, V: Serializable>(&mut self, key: K, value: V) {
        self.insert_boxed(Box::new(key), Box::new(value));
    }

    pub fn insert_boxed(&mut self, key: Box<dyn Serializable>, value: Box<dyn Serializable>) {
        match self
            .indices
            .entry(key.compute_hash().expect(HASH_ERROR_MESSAGE))
        {
            Entry::Occupied(entry) => {
                self.values[*entry.get()] = (key, value);
            }
            Entry::Vacant(entry) => {
                entry.insert(self.values.len());
                self.values.push((key, value));
            }
        }
    }
}

impl SerializableMap for DynamicSerializableMap {
    fn get_with_key(&self, key: &dyn Serializable) -> Option<&dyn Serializable> {
        self.indices
            .get(&key.compute_hash().expect(HASH_ERROR_MESSAGE))
            .map(|index| &*self.values.get(*index).unwrap().1)
    }

    fn get_with_key_mut(&mut self, key: &dyn Serializable) -> Option<&mut dyn Serializable> {
        self.indices
            .get(&key.compute_hash().expect(HASH_ERROR_MESSAGE))
            .cloned()
            .map(move |index| &mut *self.values[index].1)
    }

    fn count(&self) -> usize {
        self.values.len()
    }

    fn clone_as_dynamic(&self) -> DynamicSerializableMap {
        DynamicSerializableMap {
            name: self.name.clone(),
            values: self
                .values
                .iter()
                .map(|(key, value)| (key.duplicate(), value.duplicate()))
                .collect(),
            indices: self.indices.clone(),
        }
    }

    fn iter_serializable(&self) -> SerializableMapIterator {
        SerializableMapIterator {
            map: self,
            index: 0,
        }
    }

    fn get_at(&self, index: usize) -> Option<(&dyn Serializable, &dyn Serializable)> {
        self.values
            .get(index)
            .map(|(key, value)| (&**key, &**value))
    }
}

impl Serializable for DynamicSerializableMap {
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
    fn set_from(&mut self, value: &dyn Serializable, registry: &SerializableRegistry) {
        if let SerializableRef::Map(map_value) = value.serializable_ref() {
            for (key, value) in map_value.iter_serializable() {
                if let Some(v) = self.get_with_key_mut(key) {
                    v.set_from(value, registry)
                } else {
                    self.insert_boxed(key.duplicate(), value.duplicate());
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
        map_hash(self)
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

pub struct SerializableMapIterator<'a> {
    pub(crate) map: &'a dyn SerializableMap,
    pub(crate) index: usize,
}

impl<'a> Iterator for SerializableMapIterator<'a> {
    type Item = (&'a dyn Serializable, &'a dyn Serializable);

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.map.get_at(self.index);
        self.index += 1;
        value
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.map.count();
        (size, Some(size))
    }
}

impl<'a> ExactSizeIterator for SerializableMapIterator<'a> {}

#[inline]
pub fn map_hash<T>(map: &T) -> Option<u64>
where
    T: SerializableMap,
{
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    std::any::Any::type_id(map).hash(&mut hasher);
    map.count().hash(&mut hasher);
    for (_, value) in map.iter_serializable() {
        hasher.write_u64(value.compute_hash()?)
    }
    Some(hasher.finish())
}

#[inline]
pub fn is_map_equal<T>(a: &T, b: &dyn Serializable) -> bool
where
    T: SerializableMap,
{
    let map = if let SerializableRef::Map(map) = b.serializable_ref() {
        map
    } else {
        return false;
    };

    if a.count() != map.count() {
        return false;
    }

    for (key, value) in a.iter_serializable() {
        if let Some(map_value) = map.get_with_key(key) {
            if let Some(false) | None = value.is_equal(map_value) {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}
