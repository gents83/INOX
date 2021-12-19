use crate::SerializableRegistry;
use crate::{
    serialization::SerializableValue, SerializableArray, SerializableList, SerializableMap,
    SerializableStruct, SerializableTuple, SerializableTupleStruct,
};
use std::any::Any;
use std::fmt::Debug;

pub enum SerializableRef<'a> {
    Box(&'a dyn Serializable),
    Struct(&'a dyn SerializableStruct),
    TupleStruct(&'a dyn SerializableTupleStruct),
    Tuple(&'a dyn SerializableTuple),
    Array(&'a dyn SerializableArray),
    List(&'a dyn SerializableList),
    Map(&'a dyn SerializableMap),
    Value(&'a dyn Serializable),
}

pub enum SerializableMut<'a> {
    Box(&'a mut dyn Serializable),
    Struct(&'a mut dyn SerializableStruct),
    TupleStruct(&'a mut dyn SerializableTupleStruct),
    Tuple(&'a mut dyn SerializableTuple),
    Array(&'a mut dyn SerializableArray),
    List(&'a mut dyn SerializableList),
    Map(&'a mut dyn SerializableMap),
    Value(&'a mut dyn Serializable),
}

pub trait Serializable: Any + Send + Sync {
    fn type_name(&self) -> String;
    fn as_serializable(&self) -> &dyn Serializable;
    fn as_serializable_mut(&mut self) -> &mut dyn Serializable;
    fn any(&self) -> &dyn Any;
    fn any_mut(&mut self) -> &mut dyn Any;
    fn set_from(&mut self, value: &dyn Serializable, registry: &SerializableRegistry);
    fn duplicate(&self) -> Box<dyn Serializable>;
    fn compute_hash(&self) -> Option<u64>;
    fn is_equal(&self, _value: &dyn Serializable) -> Option<bool>;
    fn serializable_ref(&self) -> SerializableRef;
    fn serializable_mut(&mut self) -> SerializableMut;
    fn serializable_value(&self) -> Option<SerializableValue>;
}

pub trait FromSerializable: Serializable + Send + Sync + Sized {
    fn from_serializable(value: &dyn Serializable, registry: &SerializableRegistry)
        -> Option<Self>;
}
pub trait AsAny: Any + Send + Sync {
    fn as_any(&self) -> Box<dyn Any>;
}

impl Debug for dyn Serializable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Serializable[{}]", self.type_name())
    }
}

impl dyn Serializable {
    pub fn downcast<T>(self: Box<dyn Serializable>) -> Result<Box<T>, Box<dyn Serializable>>
    where
        T: Serializable,
    {
        if self.is::<T>() {
            unsafe {
                let raw: *mut dyn Serializable = Box::into_raw(self);
                Ok(Box::from_raw(raw as *mut T))
            }
        } else {
            Err(self)
        }
    }

    #[inline]
    pub fn is<T>(&self) -> bool
    where
        T: Serializable,
    {
        self.any().is::<T>()
    }

    #[inline]
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: Serializable,
    {
        self.any().downcast_ref::<T>()
    }

    #[inline]
    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Serializable,
    {
        self.any_mut().downcast_mut::<T>()
    }

    pub fn take<T>(self: Box<dyn Serializable>) -> Result<T, Box<dyn Serializable>>
    where
        T: Serializable,
    {
        self.downcast::<T>().map(|value| *value)
    }
}
