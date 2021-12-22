use crate::{
    serialization::serializable_types, Serializable, SerializableArray, SerializableList,
    SerializableMap, SerializableRef, SerializableRegistry, SerializableStruct, SerializableTuple,
    SerializableTupleStruct,
};
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize,
};

pub enum SerializableValue<'a> {
    Boxed(Box<dyn erased_serde::Serialize + 'a>),
    Ref(&'a dyn erased_serde::Serialize),
}

impl<'a> SerializableValue<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn borrow(&self) -> &dyn erased_serde::Serialize {
        match self {
            SerializableValue::Ref(serialize) => serialize,
            SerializableValue::Boxed(serialize) => serialize,
        }
    }
}

fn get_serializable<E: serde::ser::Error>(
    value: &dyn Serializable,
) -> Result<SerializableValue, E> {
    value.serializable_value().ok_or_else(|| {
        serde::ser::Error::custom(format_args!(
            "Type '{}' does not support `Serializable` serialization",
            value.type_name().as_str()
        ))
    })
}

pub struct SerializableSerializer<'a> {
    pub value: &'a dyn Serializable,
    pub registry: &'a SerializableRegistry,
}

impl<'a> SerializableSerializer<'a> {
    pub fn new(value: &'a dyn Serializable, registry: &'a SerializableRegistry) -> Self {
        SerializableSerializer { value, registry }
    }
}

impl<'a> Serialize for SerializableSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.value.serializable_ref() {
            SerializableRef::Struct(value) => StructSerializer {
                struct_value: value,
                registry: self.registry,
            }
            .serialize(serializer),
            SerializableRef::TupleStruct(value) => TupleStructSerializer {
                tuple_struct: value,
                registry: self.registry,
            }
            .serialize(serializer),
            SerializableRef::Tuple(value) => TupleSerializer {
                tuple: value,
                registry: self.registry,
            }
            .serialize(serializer),
            SerializableRef::Array(value) => ArraySerializer {
                array: value,
                registry: self.registry,
            }
            .serialize(serializer),
            SerializableRef::List(value) => ListSerializer {
                list: value,
                registry: self.registry,
            }
            .serialize(serializer),
            SerializableRef::Map(value) => MapSerializer {
                map: value,
                registry: self.registry,
            }
            .serialize(serializer),
            SerializableRef::Value(value) => SerializableValueSerializer {
                registry: self.registry,
                value,
            }
            .serialize(serializer),
        }
    }
}

pub struct SerializableValueSerializer<'a> {
    pub registry: &'a SerializableRegistry,
    pub value: &'a dyn Serializable,
}

impl<'a> Serialize for SerializableValueSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(2))?;
        state.serialize_entry(serializable_types::TYPE, self.value.type_name().as_str())?;
        state.serialize_entry(
            serializable_types::VALUE,
            get_serializable::<S::Error>(self.value)?.borrow(),
        )?;
        state.end()
    }
}

pub struct StructSerializer<'a> {
    pub struct_value: &'a dyn SerializableStruct,
    pub registry: &'a SerializableRegistry,
}

impl<'a> Serialize for StructSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(2))?;

        state.serialize_entry(
            serializable_types::TYPE,
            self.struct_value.type_name().as_str(),
        )?;
        state.serialize_entry(
            serializable_types::STRUCT,
            &StructValueSerializer {
                struct_value: self.struct_value,
                registry: self.registry,
            },
        )?;
        state.end()
    }
}

pub struct StructValueSerializer<'a> {
    pub struct_value: &'a dyn SerializableStruct,
    pub registry: &'a SerializableRegistry,
}

impl<'a> Serialize for StructValueSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.struct_value.fields_count()))?;
        for (index, value) in self.struct_value.iter_fields().enumerate() {
            let key = self.struct_value.name_at(index).unwrap();
            state.serialize_entry(key, &SerializableSerializer::new(value, self.registry))?;
        }
        state.end()
    }
}

pub struct TupleStructSerializer<'a> {
    pub tuple_struct: &'a dyn SerializableTupleStruct,
    pub registry: &'a SerializableRegistry,
}

impl<'a> Serialize for TupleStructSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(2))?;

        state.serialize_entry(
            serializable_types::TYPE,
            self.tuple_struct.type_name().as_str(),
        )?;
        state.serialize_entry(
            serializable_types::TUPLE_STRUCT,
            &TupleStructValueSerializer {
                tuple_struct: self.tuple_struct,
                registry: self.registry,
            },
        )?;
        state.end()
    }
}

pub struct TupleStructValueSerializer<'a> {
    pub tuple_struct: &'a dyn SerializableTupleStruct,
    pub registry: &'a SerializableRegistry,
}

impl<'a> Serialize for TupleStructValueSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.tuple_struct.fields_count()))?;
        for value in self.tuple_struct.iter_fields() {
            state.serialize_element(&SerializableSerializer::new(value, self.registry))?;
        }
        state.end()
    }
}

pub struct TupleSerializer<'a> {
    pub tuple: &'a dyn SerializableTuple,
    pub registry: &'a SerializableRegistry,
}

impl<'a> Serialize for TupleSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(2))?;

        state.serialize_entry(serializable_types::TYPE, self.tuple.type_name().as_str())?;
        state.serialize_entry(
            serializable_types::TUPLE,
            &TupleValueSerializer {
                tuple: self.tuple,
                registry: self.registry,
            },
        )?;
        state.end()
    }
}

pub struct TupleValueSerializer<'a> {
    pub tuple: &'a dyn SerializableTuple,
    pub registry: &'a SerializableRegistry,
}

impl<'a> Serialize for TupleValueSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.tuple.fields_count()))?;
        for value in self.tuple.iter_fields() {
            state.serialize_element(&SerializableSerializer::new(value, self.registry))?;
        }
        state.end()
    }
}

pub struct MapSerializer<'a> {
    pub map: &'a dyn SerializableMap,
    pub registry: &'a SerializableRegistry,
}

impl<'a> Serialize for MapSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(2))?;

        state.serialize_entry(serializable_types::TYPE, self.map.type_name().as_str())?;
        state.serialize_entry(
            serializable_types::MAP,
            &MapValueSerializer {
                map: self.map,
                registry: self.registry,
            },
        )?;
        state.end()
    }
}

pub struct MapValueSerializer<'a> {
    pub map: &'a dyn SerializableMap,
    pub registry: &'a SerializableRegistry,
}

impl<'a> Serialize for MapValueSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.map.count() * 2))?;
        for (key, value) in self.map.iter_serializable() {
            state.serialize_element(&SerializableSerializer::new(key, self.registry))?;
            state.serialize_element(&SerializableSerializer::new(value, self.registry))?;
        }
        state.end()
        /*
        let mut state = serializer.serialize_map(Some(self.map.count()))?;
        for (key, value) in self.map.iter_serializable() {
            let k = SerializableSerializer::new(key, self.registry);
            let v = SerializableSerializer::new(value, self.registry);
            state.serialize_entry(serializable_types::KEY, &k)?;
            state.serialize_entry(serializable_types::VALUE, &v)?;
        }
        state.end()
        */
    }
}

pub struct ListSerializer<'a> {
    pub list: &'a dyn SerializableList,
    pub registry: &'a SerializableRegistry,
}

impl<'a> Serialize for ListSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(2))?;
        state.serialize_entry(serializable_types::TYPE, self.list.type_name().as_str())?;
        state.serialize_entry(
            serializable_types::LIST,
            &ListValueSerializer {
                list: self.list,
                registry: self.registry,
            },
        )?;
        state.end()
    }
}

pub struct ListValueSerializer<'a> {
    pub list: &'a dyn SerializableList,
    pub registry: &'a SerializableRegistry,
}

impl<'a> Serialize for ListValueSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.list.count()))?;
        for value in self.list.iter_serializable() {
            state.serialize_element(&SerializableSerializer::new(value, self.registry))?;
        }
        state.end()
    }
}

pub struct ArraySerializer<'a> {
    pub array: &'a dyn SerializableArray,
    pub registry: &'a SerializableRegistry,
}

impl<'a> Serialize for ArraySerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(2))?;
        state.serialize_entry(serializable_types::TYPE, self.array.type_name().as_str())?;
        state.serialize_entry(
            serializable_types::ARRAY,
            &ArrayValueSerializer {
                array: self.array,
                registry: self.registry,
            },
        )?;
        state.end()
    }
}

pub struct ArrayValueSerializer<'a> {
    pub array: &'a dyn SerializableArray,
    pub registry: &'a SerializableRegistry,
}

impl<'a> Serialize for ArrayValueSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.array.count()))?;
        for value in self.array.iter_serializable() {
            state.serialize_element(&SerializableSerializer::new(value, self.registry))?;
        }
        state.end()
    }
}
