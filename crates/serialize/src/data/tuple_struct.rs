use crate::{
    serialization::SerializableValue, Serializable, SerializableMut, SerializableRef,
    SerializableRegistry,
};
use std::any::Any;

/// A rust "tuple struct" reflection
pub trait SerializableTupleStruct: Serializable {
    fn field(&self, index: usize) -> Option<&dyn Serializable>;
    fn field_mut(&mut self, index: usize) -> Option<&mut dyn Serializable>;
    fn fields_count(&self) -> usize;
    fn iter_fields(&self) -> SerializableTupleStructFieldIterator;
    fn clone_as_dynamic(&self) -> SerializableDynamicTupleStruct;
}

pub struct SerializableTupleStructFieldIterator<'a> {
    pub(crate) tuple_struct: &'a dyn SerializableTupleStruct,
    pub(crate) index: usize,
}

impl<'a> SerializableTupleStructFieldIterator<'a> {
    pub fn new(value: &'a dyn SerializableTupleStruct) -> Self {
        SerializableTupleStructFieldIterator {
            tuple_struct: value,
            index: 0,
        }
    }
}

impl<'a> Iterator for SerializableTupleStructFieldIterator<'a> {
    type Item = &'a dyn Serializable;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.tuple_struct.field(self.index);
        self.index += 1;
        value
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.tuple_struct.fields_count();
        (size, Some(size))
    }
}

impl<'a> ExactSizeIterator for SerializableTupleStructFieldIterator<'a> {}

pub trait GetTupleStructField {
    fn get_field<T: Serializable>(&self, index: usize) -> Option<&T>;
    fn get_field_mut<T: Serializable>(&mut self, index: usize) -> Option<&mut T>;
}

impl<S> GetTupleStructField for S
where
    S: SerializableTupleStruct,
{
    fn get_field<T>(&self, index: usize) -> Option<&T>
    where
        T: Serializable,
    {
        self.field(index)
            .and_then(|value| value.downcast_ref::<T>())
    }

    fn get_field_mut<T>(&mut self, index: usize) -> Option<&mut T>
    where
        T: Serializable,
    {
        self.field_mut(index)
            .and_then(|value| value.downcast_mut::<T>())
    }
}

impl GetTupleStructField for dyn SerializableTupleStruct {
    fn get_field<T>(&self, index: usize) -> Option<&T>
    where
        T: Serializable,
    {
        self.field(index)
            .and_then(|value| value.downcast_ref::<T>())
    }

    fn get_field_mut<T>(&mut self, index: usize) -> Option<&mut T>
    where
        T: Serializable,
    {
        self.field_mut(index)
            .and_then(|value| value.downcast_mut::<T>())
    }
}

#[derive(Default)]
pub struct SerializableDynamicTupleStruct {
    name: String,
    fields: Vec<Box<dyn Serializable>>,
}

impl SerializableDynamicTupleStruct {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn insert_boxed(&mut self, value: Box<dyn Serializable>) {
        self.fields.push(value);
    }

    pub fn insert<T: Serializable>(&mut self, value: T) {
        self.insert_boxed(Box::new(value));
    }
}

impl SerializableTupleStruct for SerializableDynamicTupleStruct {
    #[inline]
    fn field(&self, index: usize) -> Option<&dyn Serializable> {
        self.fields.get(index).map(|field| &**field)
    }

    #[inline]
    fn field_mut(&mut self, index: usize) -> Option<&mut dyn Serializable> {
        self.fields.get_mut(index).map(|field| &mut **field)
    }

    #[inline]
    fn fields_count(&self) -> usize {
        self.fields.len()
    }

    #[inline]
    fn iter_fields(&self) -> SerializableTupleStructFieldIterator {
        SerializableTupleStructFieldIterator {
            tuple_struct: self,
            index: 0,
        }
    }

    fn clone_as_dynamic(&self) -> SerializableDynamicTupleStruct {
        SerializableDynamicTupleStruct {
            name: self.name.clone(),
            fields: self.fields.iter().map(|value| value.duplicate()).collect(),
        }
    }
}

impl Serializable for SerializableDynamicTupleStruct {
    #[inline]
    fn type_name(&self) -> String {
        self.name.clone()
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
    fn duplicate(&self) -> Box<dyn Serializable> {
        Box::new(self.clone_as_dynamic())
    }

    #[inline]
    fn serializable_ref(&self) -> SerializableRef {
        SerializableRef::TupleStruct(self)
    }

    #[inline]
    fn serializable_mut(&mut self) -> SerializableMut {
        SerializableMut::TupleStruct(self)
    }

    #[inline]
    fn set_from(&mut self, value: &dyn Serializable, registry: &SerializableRegistry) {
        if let SerializableRef::TupleStruct(tuple_struct) = value.serializable_ref() {
            for (i, value) in tuple_struct.iter_fields().enumerate() {
                if let Some(v) = self.field_mut(i) {
                    v.set_from(value, registry)
                }
            }
        } else {
            panic!("Attempted to apply non-TupleStruct type to TupleStruct type.");
        }
    }

    #[inline]
    fn compute_hash(&self) -> Option<u64> {
        tuple_struct_hash(self)
    }

    #[inline]
    fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
        Some(is_tuple_struct_equal(self, value))
    }

    #[inline]
    fn serializable_value(&self) -> Option<SerializableValue> {
        None
    }
}

#[inline]
pub fn tuple_struct_hash<T>(s: &T) -> Option<u64>
where
    T: SerializableTupleStruct,
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
pub fn is_tuple_struct_equal<T>(a: &T, b: &dyn Serializable) -> bool
where
    T: SerializableTupleStruct,
{
    let tuple_struct = if let SerializableRef::TupleStruct(tuple_struct) = b.serializable_ref() {
        tuple_struct
    } else {
        return false;
    };

    if a.fields_count() != tuple_struct.fields_count() {
        return false;
    }

    for (i, value) in tuple_struct.iter_fields().enumerate() {
        if let Some(field_value) = a.field(i) {
            if let Some(false) | None = field_value.is_equal(value) {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}
