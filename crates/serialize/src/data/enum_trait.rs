use std::{
    any::Any,
    borrow::Cow,
    collections::{hash_map::Entry, HashMap},
};

use crate::{
    Serializable, SerializableMut, SerializableRef, SerializableRegistry, SerializableStruct,
    SerializableTuple, SerializableValue,
};

pub trait SerializableEnum: Serializable {
    fn variant(&self) -> SerializableEnumVariant<'_>;
    fn variant_mut(&mut self) -> SerializableEnumVariantMut<'_>;
    fn variant_info(&self) -> SerializableVariantInfo<'_>;
    fn iter_variants_info(&self) -> SerializableVariantInfoIterator<'_>;
    fn get_index_name(&self, index: usize) -> Option<&str>;
    fn get_index_from_name(&self, name: &str) -> Option<usize>;
    fn clone_as_dynamic(&self) -> SerializableDynamicEnum;
}
#[derive(PartialEq, Eq)]
pub struct SerializableVariantInfo<'a> {
    pub index: usize,
    pub name: &'a str,
}
pub struct SerializableVariantInfoIterator<'a> {
    pub(crate) value: &'a dyn SerializableEnum,
    pub(crate) index: usize,
}
impl<'a> SerializableVariantInfoIterator<'a> {
    pub fn new(value: &'a dyn SerializableEnum) -> Self {
        Self { value, index: 0 }
    }
}
impl<'a> Iterator for SerializableVariantInfoIterator<'a> {
    type Item = SerializableVariantInfo<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self
            .value
            .get_index_name(self.index)
            .map(|name| SerializableVariantInfo {
                index: self.index,
                name,
            });
        self.index += 1;
        value
    }
}

pub enum SerializableEnumVariant<'a> {
    Unit,
    NewType(&'a dyn Serializable),
    Tuple(&'a dyn SerializableTuple),
    Struct(&'a dyn SerializableStruct),
}
pub enum SerializableEnumVariantMut<'a> {
    Unit,
    NewType(&'a mut dyn Serializable),
    Tuple(&'a mut dyn SerializableTuple),
    Struct(&'a mut dyn SerializableStruct),
}

#[derive(Default)]
pub struct SerializableDynamicEnum {
    name: String,
    variant: usize,
    variants: Vec<Box<dyn Serializable>>,
    variants_names: Vec<Cow<'static, str>>,
    variants_indices: HashMap<Cow<'static, str>, usize>,
}

impl SerializableDynamicEnum {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_variant_index(&mut self, index: usize) {
        self.variant = index;
    }

    pub fn insert_boxed(&mut self, name: &str, value: Box<dyn Serializable>) {
        let name = Cow::Owned(name.to_string());
        match self.variants_indices.entry(name) {
            Entry::Occupied(entry) => {
                self.variants[*entry.get()] = value;
            }
            Entry::Vacant(entry) => {
                self.variants.push(value);
                self.variants_names.push(entry.key().clone());
                entry.insert(self.variants.len() - 1);
            }
        }
    }

    pub fn insert<T: Serializable>(&mut self, name: &str, value: T) {
        if let Some(index) = self.variants_indices.get(name) {
            self.variants[*index] = Box::new(value);
        } else {
            self.insert_boxed(name, Box::new(value));
        }
    }
}

impl SerializableEnum for SerializableDynamicEnum {
    fn variant(&self) -> SerializableEnumVariant<'_> {
        match self.variants[self.variant].serializable_ref() {
            SerializableRef::Struct(value) => SerializableEnumVariant::Struct(value),
            SerializableRef::Tuple(value) => SerializableEnumVariant::Tuple(value),
            SerializableRef::Array(value) => {
                SerializableEnumVariant::NewType(value.as_serializable())
            }
            SerializableRef::List(value) => {
                SerializableEnumVariant::NewType(value.as_serializable())
            }
            SerializableRef::Map(value) => {
                SerializableEnumVariant::NewType(value.as_serializable())
            }
            SerializableRef::TupleStruct(value) => {
                SerializableEnumVariant::NewType(value.as_serializable())
            }
            SerializableRef::Value(value) => SerializableEnumVariant::NewType(value),
            _ => SerializableEnumVariant::Unit,
        }
    }

    fn variant_mut(&mut self) -> SerializableEnumVariantMut<'_> {
        match self.variants[self.variant].serializable_mut() {
            SerializableMut::Struct(value) => SerializableEnumVariantMut::Struct(value),
            SerializableMut::Tuple(value) => SerializableEnumVariantMut::Tuple(value),
            SerializableMut::Array(value) => {
                SerializableEnumVariantMut::NewType(value.as_serializable_mut())
            }
            SerializableMut::List(value) => {
                SerializableEnumVariantMut::NewType(value.as_serializable_mut())
            }
            SerializableMut::Map(value) => {
                SerializableEnumVariantMut::NewType(value.as_serializable_mut())
            }
            SerializableMut::TupleStruct(value) => {
                SerializableEnumVariantMut::NewType(value.as_serializable_mut())
            }
            SerializableMut::Value(value) => SerializableEnumVariantMut::NewType(value),
            _ => SerializableEnumVariantMut::Unit,
        }
    }

    fn variant_info(&self) -> SerializableVariantInfo<'_> {
        SerializableVariantInfo {
            index: self.variant,
            name: &self.variants_names[self.variant],
        }
    }

    fn iter_variants_info(&self) -> SerializableVariantInfoIterator<'_> {
        SerializableVariantInfoIterator::new(self)
    }

    fn get_index_name(&self, index: usize) -> Option<&str> {
        self.variants_names.get(index).map(|name| name.as_ref())
    }

    fn get_index_from_name(&self, name: &str) -> Option<usize> {
        self.variants_indices.get(name).copied()
    }

    fn clone_as_dynamic(&self) -> SerializableDynamicEnum {
        SerializableDynamicEnum {
            name: self.name.clone(),
            variant: 0usize,
            variants_names: self.variants_names.clone(),
            variants_indices: self.variants_indices.clone(),
            variants: self
                .variants
                .iter()
                .map(|value| value.duplicate())
                .collect(),
        }
    }
}

impl Serializable for SerializableDynamicEnum {
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
        SerializableRef::Enum(self)
    }

    #[inline]
    fn serializable_mut(&mut self) -> SerializableMut {
        SerializableMut::Enum(self)
    }

    #[inline]
    fn set(&mut self, value: &dyn Serializable, registry: &SerializableRegistry) {
        if let SerializableRef::Enum(enum_value) = value.serializable_ref() {
            let info = enum_value.variant_info();
            if info.index < self.variants.len() {
                self.variants[info.index].as_mut().set(value, registry);
            } else {
                panic!("Invalid variant index");
            }
        } else {
            panic!("Attempted to apply non-enum type to enum type.");
        }
    }

    #[inline]
    fn compute_hash(&self) -> Option<u64> {
        None
    }

    #[inline]
    fn is_equal(&self, value: &dyn Serializable) -> Option<bool> {
        Some(is_enum_equal(self, value))
    }

    #[inline]
    fn serializable_value(&self) -> Option<SerializableValue> {
        None
    }
}

#[inline]
pub fn is_enum_equal<E: SerializableEnum>(enum_a: &E, v: &dyn Serializable) -> bool {
    let enum_b = if let SerializableRef::Enum(e) = v.serializable_ref() {
        e
    } else {
        return false;
    };

    if enum_a.variant_info() != enum_b.variant_info() {
        return false;
    }

    let variant_b = enum_b.variant();
    match enum_a.variant() {
        SerializableEnumVariant::Unit => {
            if let SerializableEnumVariant::Unit = variant_b {
            } else {
                return false;
            }
        }
        SerializableEnumVariant::NewType(t_a) => {
            if let SerializableEnumVariant::NewType(t_b) = variant_b {
                if let Some(false) | None = t_b.is_equal(t_a) {
                    return false;
                }
            } else {
                return false;
            }
        }
        SerializableEnumVariant::Tuple(t_a) => {
            if let SerializableEnumVariant::Tuple(t_b) = variant_b {
                if let Some(false) | None = t_b.is_equal(t_a.as_serializable()) {
                    return false;
                }
            } else {
                return false;
            }
        }
        SerializableEnumVariant::Struct(s_a) => {
            if let SerializableEnumVariant::Struct(s_b) = variant_b {
                if let Some(false) | None = s_b.is_equal(s_a.as_serializable()) {
                    return false;
                }
            } else {
                return false;
            }
        }
    }
    true
}
