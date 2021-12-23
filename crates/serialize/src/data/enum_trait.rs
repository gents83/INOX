use crate::{Serializable, SerializableRef, SerializableStruct, SerializableTuple};

pub trait SerializableEnum: Serializable {
    fn variant(&self) -> SerializableEnumVariant<'_>;
    fn variant_mut(&mut self) -> SerializableEnumVariantMut<'_>;
    fn variant_info(&self) -> SerializableVariantInfo<'_>;
    fn iter_variants_info(&self) -> SerializableVariantInfoIterator<'_>;
    fn get_index_name(&self, index: usize) -> Option<&str>;
    fn get_index_from_name(&self, name: &str) -> Option<usize>;
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
