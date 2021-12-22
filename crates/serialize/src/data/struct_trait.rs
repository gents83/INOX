use crate::{
    serialization::SerializableValue, Serializable, SerializableMut, SerializableRef,
    SerializableRegistry,
};
use std::{
    any::Any,
    borrow::Cow,
    collections::{hash_map::Entry, HashMap},
};

pub trait SerializableStruct: Serializable {
    fn field(&self, name: &str) -> Option<&dyn Serializable>;
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Serializable>;
    fn field_at(&self, index: usize) -> Option<&dyn Serializable>;
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Serializable>;
    fn name_at(&self, index: usize) -> Option<&str>;
    fn fields_count(&self) -> usize;
    fn iter_fields(&self) -> SerializableFieldIterator;
    fn clone_as_dynamic(&self) -> SerializableDynamicStruct;
}

pub struct SerializableFieldIterator<'a> {
    pub(crate) struct_val: &'a dyn SerializableStruct,
    pub(crate) index: usize,
}

impl<'a> SerializableFieldIterator<'a> {
    pub fn new(value: &'a dyn SerializableStruct) -> Self {
        SerializableFieldIterator {
            struct_val: value,
            index: 0,
        }
    }
}

impl<'a> Iterator for SerializableFieldIterator<'a> {
    type Item = &'a dyn Serializable;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.struct_val.field_at(self.index);
        self.index += 1;
        value
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.struct_val.fields_count();
        (size, Some(size))
    }
}

impl<'a> ExactSizeIterator for SerializableFieldIterator<'a> {}

pub trait GetField {
    fn get_field<T: Serializable>(&self, name: &str) -> Option<&T>;
    fn get_field_mut<T: Serializable>(&mut self, name: &str) -> Option<&mut T>;
}

impl<S: SerializableStruct> GetField for S {
    fn get_field<T: Serializable>(&self, name: &str) -> Option<&T> {
        self.field(name).and_then(|value| value.downcast_ref::<T>())
    }

    fn get_field_mut<T: Serializable>(&mut self, name: &str) -> Option<&mut T> {
        self.field_mut(name)
            .and_then(|value| value.downcast_mut::<T>())
    }
}

impl GetField for dyn SerializableStruct {
    fn get_field<T: Serializable>(&self, name: &str) -> Option<&T> {
        self.field(name).and_then(|value| value.downcast_ref::<T>())
    }

    fn get_field_mut<T: Serializable>(&mut self, name: &str) -> Option<&mut T> {
        self.field_mut(name)
            .and_then(|value| value.downcast_mut::<T>())
    }
}

#[derive(Default)]
pub struct SerializableDynamicStruct {
    name: String,
    fields: Vec<Box<dyn Serializable>>,
    field_names: Vec<Cow<'static, str>>,
    field_indices: HashMap<Cow<'static, str>, usize>,
}

impl SerializableDynamicStruct {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn insert_boxed(&mut self, name: &str, value: Box<dyn Serializable>) {
        let name = Cow::Owned(name.to_string());
        match self.field_indices.entry(name) {
            Entry::Occupied(entry) => {
                self.fields[*entry.get()] = value;
            }
            Entry::Vacant(entry) => {
                self.fields.push(value);
                self.field_names.push(entry.key().clone());
                entry.insert(self.fields.len() - 1);
            }
        }
    }

    pub fn insert<T: Serializable>(&mut self, name: &str, value: T) {
        if let Some(index) = self.field_indices.get(name) {
            self.fields[*index] = Box::new(value);
        } else {
            self.insert_boxed(name, Box::new(value));
        }
    }
}

impl SerializableStruct for SerializableDynamicStruct {
    #[inline]
    fn field(&self, name: &str) -> Option<&dyn Serializable> {
        self.field_indices
            .get(name)
            .map(|index| &*self.fields[*index])
    }

    #[inline]
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Serializable> {
        if let Some(index) = self.field_indices.get(name) {
            Some(&mut *self.fields[*index])
        } else {
            None
        }
    }

    #[inline]
    fn field_at(&self, index: usize) -> Option<&dyn Serializable> {
        self.fields.get(index).map(|value| &**value)
    }

    #[inline]
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Serializable> {
        self.fields.get_mut(index).map(|value| &mut **value)
    }

    #[inline]
    fn name_at(&self, index: usize) -> Option<&str> {
        self.field_names.get(index).map(|name| name.as_ref())
    }

    #[inline]
    fn fields_count(&self) -> usize {
        self.fields.len()
    }

    #[inline]
    fn iter_fields(&self) -> SerializableFieldIterator {
        SerializableFieldIterator {
            struct_val: self,
            index: 0,
        }
    }

    fn clone_as_dynamic(&self) -> SerializableDynamicStruct {
        SerializableDynamicStruct {
            name: self.name.clone(),
            field_names: self.field_names.clone(),
            field_indices: self.field_indices.clone(),
            fields: self.fields.iter().map(|value| value.duplicate()).collect(),
        }
    }
}

impl Serializable for SerializableDynamicStruct {
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
    fn duplicate(&self) -> Box<dyn Serializable> {
        Box::new(self.clone_as_dynamic())
    }

    #[inline]
    fn serializable_ref(&self) -> SerializableRef {
        SerializableRef::Struct(self)
    }

    #[inline]
    fn serializable_mut(&mut self) -> SerializableMut {
        SerializableMut::Struct(self)
    }

    #[inline]
    fn set(&mut self, value: &dyn Serializable, registry: &SerializableRegistry) {
        if let SerializableRef::Struct(struct_value) = value.serializable_ref() {
            for (i, value) in struct_value.iter_fields().enumerate() {
                let name = struct_value.name_at(i).unwrap();
                if let Some(v) = self.field_mut(name) {
                    v.set(value, registry)
                }
            }
        } else {
            panic!("Attempted to apply non-struct type to struct type.");
        }
    }

    #[inline]
    fn compute_hash(&self) -> Option<u64> {
        struct_hash(self)
    }

    #[inline]
    fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
        Some(is_struct_equal(self, value))
    }

    #[inline]
    fn serializable_value(&self) -> Option<SerializableValue> {
        None
    }
}

#[inline]
pub fn struct_hash<T>(s: &T) -> Option<u64>
where
    T: SerializableStruct,
{
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    std::any::Any::type_id(s).hash(&mut hasher);
    s.fields_count().hash(&mut hasher);
    for value in s.iter_fields() {
        hasher.write_u64(value.compute_hash()?)
    }
    Some(hasher.finish())
}

#[inline]
pub fn is_struct_equal<S: SerializableStruct>(a: &S, b: &dyn Serializable) -> bool {
    let struct_value = if let SerializableRef::Struct(struct_value) = b.serializable_ref() {
        struct_value
    } else {
        return false;
    };

    if a.fields_count() != struct_value.fields_count() {
        return false;
    }

    for (i, value) in struct_value.iter_fields().enumerate() {
        let name = struct_value.name_at(i).unwrap();
        if let Some(field_value) = a.field(name) {
            if let Some(false) | None = field_value.is_equal(value) {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}
