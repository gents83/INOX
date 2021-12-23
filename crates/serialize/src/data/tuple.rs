use std::any::Any;

use serde::ser::SerializeSeq;

use crate::{
    serialization::SerializableValue, FromSerializable, Serializable, SerializableMut,
    SerializableRef, SerializableRegistry,
};

pub trait SerializableTuple: Serializable {
    fn field(&self, index: usize) -> Option<&dyn Serializable>;
    fn field_mut(&mut self, index: usize) -> Option<&mut dyn Serializable>;
    fn fields_count(&self) -> usize;
    fn iter_fields(&self) -> SerializableTupleFieldIterator;
    fn clone_as_dynamic(&self) -> SerializableDynamicTuple;
}

pub struct AsSerializableTuple<'a>(pub &'a dyn SerializableTuple);

impl<'a> serde::Serialize for AsSerializableTuple<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::tuple_serialize(self.0, serializer)
    }
}

pub struct SerializableTupleFieldIterator<'a> {
    pub(crate) tuple: &'a dyn SerializableTuple,
    pub(crate) index: usize,
}

impl<'a> SerializableTupleFieldIterator<'a> {
    pub fn new(value: &'a dyn SerializableTuple) -> Self {
        SerializableTupleFieldIterator {
            tuple: value,
            index: 0,
        }
    }
}

impl<'a> Iterator for SerializableTupleFieldIterator<'a> {
    type Item = &'a dyn Serializable;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.tuple.field(self.index);
        self.index += 1;
        value
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.tuple.fields_count();
        (size, Some(size))
    }
}

impl<'a> ExactSizeIterator for SerializableTupleFieldIterator<'a> {}

impl<'a> serde::Serialize for dyn SerializableTuple {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        tuple_serialize(self, serializer)
    }
}

impl serde::Serialize for SerializableDynamicTuple {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        tuple_serialize(self, serializer)
    }
}

#[inline]
pub fn tuple_serialize<A, S>(tuple: &A, serializer: S) -> Result<S::Ok, S::Error>
where
    A: SerializableTuple + ?Sized,
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(tuple.fields_count()))?;
    for element in tuple.iter_fields() {
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

pub trait GetTupleField {
    fn get_field<T: Serializable>(&self, index: usize) -> Option<&T>;
    fn get_field_mut<T: Serializable>(&mut self, index: usize) -> Option<&mut T>;
}

impl<S: SerializableTuple> GetTupleField for S {
    fn get_field<T: Serializable>(&self, index: usize) -> Option<&T> {
        self.field(index)
            .and_then(|value| value.downcast_ref::<T>())
    }

    fn get_field_mut<T: Serializable>(&mut self, index: usize) -> Option<&mut T> {
        self.field_mut(index)
            .and_then(|value| value.downcast_mut::<T>())
    }
}

impl GetTupleField for dyn SerializableTuple {
    fn get_field<T: Serializable>(&self, index: usize) -> Option<&T> {
        self.field(index)
            .and_then(|value| value.downcast_ref::<T>())
    }

    fn get_field_mut<T: Serializable>(&mut self, index: usize) -> Option<&mut T> {
        self.field_mut(index)
            .and_then(|value| value.downcast_mut::<T>())
    }
}

#[derive(Default)]
pub struct SerializableDynamicTuple {
    pub name: String,
    pub fields: Vec<Box<dyn Serializable>>,
}

impl SerializableDynamicTuple {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn insert_boxed(&mut self, value: Box<dyn Serializable>) {
        self.fields.push(value);
        self.generate_name();
    }

    pub fn insert<T: Serializable>(&mut self, value: T) {
        self.insert_boxed(Box::new(value));
        self.generate_name();
    }

    fn generate_name(&mut self) {
        let name = &mut self.name;
        name.clear();
        name.push('(');
        for (i, field) in self.fields.iter().enumerate() {
            if i > 0 {
                name.push_str(", ");
            }
            name.push_str(field.type_name().as_str());
        }
        name.push(')');
    }
}

impl SerializableTuple for SerializableDynamicTuple {
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
    fn iter_fields(&self) -> SerializableTupleFieldIterator {
        SerializableTupleFieldIterator {
            tuple: self,
            index: 0,
        }
    }

    #[inline]
    fn clone_as_dynamic(&self) -> SerializableDynamicTuple {
        SerializableDynamicTuple {
            name: self.name.clone(),
            fields: self.fields.iter().map(|value| value.duplicate()).collect(),
        }
    }
}

impl Serializable for SerializableDynamicTuple {
    #[inline]
    fn type_name(&self) -> String {
        self.name()
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
        SerializableRef::Tuple(self)
    }

    #[inline]
    fn serializable_mut(&mut self) -> SerializableMut {
        SerializableMut::Tuple(self)
    }

    #[inline]
    fn set(&mut self, value: &dyn Serializable, registry: &SerializableRegistry) {
        apply_in_tuple(self, value, registry);
    }

    #[inline]
    fn compute_hash(&self) -> Option<u64> {
        tuple_hash(self)
    }

    #[inline]
    fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
        Some(is_tuple_equal(self, value))
    }

    #[inline]
    fn serializable_value(&self) -> Option<SerializableValue> {
        None
    }
}

#[inline]
pub fn tuple_hash<T>(s: &T) -> Option<u64>
where
    T: SerializableTuple,
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
pub fn apply_in_tuple<T: SerializableTuple>(
    a: &mut T,
    b: &dyn Serializable,
    registry: &SerializableRegistry,
) {
    if let SerializableRef::Tuple(tuple) = b.serializable_ref() {
        for (i, value) in tuple.iter_fields().enumerate() {
            if let Some(v) = a.field_mut(i) {
                v.set(value, registry)
            }
        }
    } else {
        panic!("Attempted to apply non-Tuple type to Tuple type.");
    }
}

#[inline]
pub fn is_tuple_equal<T>(a: &T, b: &dyn Serializable) -> bool
where
    T: SerializableTuple,
{
    let b = if let SerializableRef::Tuple(tuple) = b.serializable_ref() {
        tuple
    } else {
        return false;
    };

    if a.fields_count() != b.fields_count() {
        return false;
    }

    for (a_field, b_field) in a.iter_fields().zip(b.iter_fields()) {
        match a_field.is_equal(b_field) {
            Some(false) | None => return false,
            Some(true) => {}
        }
    }

    true
}

macro_rules! impl_serializable_tuple {
    {$($index:tt : $name:tt),*} => {
        impl<$($name: Serializable),*> SerializableTuple for ($($name,)*) {
            #[inline]
            fn field(&self, index: usize) -> Option<&dyn Serializable> {
                match index {
                    $($index => Some(&self.$index as &dyn Serializable),)*
                    _ => None,
                }
            }

            #[inline]
            fn field_mut(&mut self, index: usize) -> Option<&mut dyn Serializable> {
                match index {
                    $($index => Some(&mut self.$index as &mut dyn Serializable),)*
                    _ => None,
                }
            }

            #[inline]
            fn fields_count(&self) -> usize {
                let indices: &[usize] = &[$($index as usize),*];
                indices.len()
            }

            #[inline]
            fn iter_fields(&self) -> SerializableTupleFieldIterator {
                SerializableTupleFieldIterator {
                    tuple: self,
                    index: 0,
                }
            }

            #[inline]
            fn clone_as_dynamic(&self) -> SerializableDynamicTuple {
                let mut dyn_tuple = SerializableDynamicTuple {
                    name: String::default(),
                    fields: self
                        .iter_fields()
                        .map(|value| value.duplicate())
                        .collect(),
                };
                dyn_tuple.generate_name();
                dyn_tuple
            }
        }

        impl<$($name: Serializable),*> Serializable for ($($name,)*) {
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
                crate::apply_in_tuple(self, value, registry);
            }

            #[inline]
            fn serializable_ref(&self) -> SerializableRef {
                SerializableRef::Tuple(self)
            }

            #[inline]
            fn serializable_mut(&mut self) -> SerializableMut {
                SerializableMut::Tuple(self)
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
                Some(crate::is_tuple_equal(self, value))
            }

            #[inline]
            fn serializable_value(&self) -> Option<SerializableValue> {
                None
            }
        }

        impl<$($name: FromSerializable),*> FromSerializable for ($($name,)*)
        {
            fn from_serializable(value: &dyn Serializable, _registry: &SerializableRegistry) -> Option<Self> {
                if let SerializableRef::Tuple(_ref_tuple) = value.serializable_ref() {
                    Some(
                        (
                            $(
                                <$name as FromSerializable>::from_serializable(_ref_tuple.field($index)?, _registry)?,
                            )*
                        )
                    )
                } else {
                    None
                }
            }
        }
    }
}

impl_serializable_tuple! {}
impl_serializable_tuple! {0: A}
impl_serializable_tuple! {0: A, 1: B}
impl_serializable_tuple! {0: A, 1: B, 2: C}
impl_serializable_tuple! {0: A, 1: B, 2: C, 3: D}
impl_serializable_tuple! {0: A, 1: B, 2: C, 3: D, 4: E}
impl_serializable_tuple! {0: A, 1: B, 2: C, 3: D, 4: E, 5: F}
impl_serializable_tuple! {0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G}
impl_serializable_tuple! {0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H}
impl_serializable_tuple! {0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I}
impl_serializable_tuple! {0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J}
impl_serializable_tuple! {0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K}
impl_serializable_tuple! {0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L}
