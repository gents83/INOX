use crate::{
    serialization::serializable_types, DynamicSerializableMap, Serializable,
    SerializableDeserialize, SerializableDynamicArray, SerializableDynamicList,
    SerializableDynamicStruct, SerializableDynamicTuple, SerializableDynamicTupleStruct,
    SerializableRegistry,
};
use erased_serde::Deserializer;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};

pub trait DeserializeValue {
    fn deserialize(
        deserializer: &mut dyn Deserializer,
        type_registry: &SerializableRegistry,
    ) -> Result<Box<dyn Serializable>, erased_serde::Error>;
}

pub struct SerializableDeserializer<'a> {
    registry: &'a SerializableRegistry,
}

impl<'a> SerializableDeserializer<'a> {
    pub fn new(registry: &'a SerializableRegistry) -> Self {
        SerializableDeserializer { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for SerializableDeserializer<'a> {
    type Value = Box<dyn Serializable>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(SerializableVisitor {
            registry: self.registry,
        })
    }
}

struct SerializableVisitor<'a> {
    registry: &'a SerializableRegistry,
}

impl<'a, 'de> Visitor<'de> for SerializableVisitor<'a> {
    type Value = Box<dyn Serializable>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("reflect value")
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Box::new(v))
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Box::new(v))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Box::new(v))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Box::new(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Box::new(v))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Box::new(v))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Box::new(v))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Box::new(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Box::new(v))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Box::new(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Box::new(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Box::new(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Box::new(v.to_string()))
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut type_name: Option<String> = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                serializable_types::TYPE => {
                    type_name = Some(map.next_value()?);
                }
                serializable_types::MAP => {
                    let type_name = type_name
                        .take()
                        .ok_or_else(|| de::Error::missing_field(serializable_types::TYPE))?;
                    let mut map = map.next_value_seed(MapDeserializer {
                        registry: self.registry,
                    })?;
                    map.set_name(type_name);
                    return Ok(Box::new(map));
                }
                serializable_types::STRUCT => {
                    let type_name = type_name
                        .take()
                        .ok_or_else(|| de::Error::missing_field(serializable_types::TYPE))?;
                    let mut dynamic_struct = map.next_value_seed(StructDeserializer {
                        registry: self.registry,
                    })?;
                    dynamic_struct.set_name(type_name);
                    return Ok(Box::new(dynamic_struct));
                }
                serializable_types::TUPLE_STRUCT => {
                    let type_name = type_name
                        .take()
                        .ok_or_else(|| de::Error::missing_field(serializable_types::TYPE))?;
                    let mut tuple_struct = map.next_value_seed(TupleStructDeserializer {
                        registry: self.registry,
                    })?;
                    tuple_struct.set_name(type_name);
                    return Ok(Box::new(tuple_struct));
                }
                serializable_types::TUPLE => {
                    let type_name = type_name
                        .take()
                        .ok_or_else(|| de::Error::missing_field(serializable_types::TYPE))?;
                    let mut tuple = map.next_value_seed(TupleDeserializer {
                        registry: self.registry,
                    })?;
                    tuple.set_name(type_name);
                    return Ok(Box::new(tuple));
                }
                serializable_types::LIST => {
                    let type_name = type_name
                        .take()
                        .ok_or_else(|| de::Error::missing_field(serializable_types::TYPE))?;
                    let mut list = map.next_value_seed(ListDeserializer {
                        registry: self.registry,
                    })?;
                    list.set_name(type_name);
                    return Ok(Box::new(list));
                }
                serializable_types::ARRAY => {
                    let type_name = type_name
                        .take()
                        .ok_or_else(|| de::Error::missing_field(serializable_types::TYPE))?;
                    let mut array = map.next_value_seed(ArrayDeserializer {
                        registry: self.registry,
                    })?;
                    array.set_name(type_name);
                    return Ok(Box::new(array));
                }
                serializable_types::VALUE => {
                    let type_name = type_name
                        .take()
                        .ok_or_else(|| de::Error::missing_field(serializable_types::TYPE))?;
                    let registration =
                        self.registry.get_with_fullname(&type_name).ok_or_else(|| {
                            de::Error::custom(format_args!(
                                "No registration found for {}",
                                type_name
                            ))
                        })?;
                    let deserialize_serializable = registration
                        .data::<SerializableDeserialize>()
                        .ok_or_else(|| {
                        de::Error::custom(format_args!(
                            "The TypeRegistration for {} doesn't have DeserializeSerializable",
                            type_name
                        ))
                    })?;
                    let value = map.next_value_seed(DeserializeSerializableDeserializer {
                        serializable_deserialize: deserialize_serializable,
                    })?;
                    return Ok(value);
                }
                _ => return Err(de::Error::unknown_field(key.as_str(), &[])),
            }
        }

        Err(de::Error::custom("Maps in this location must have the \'type\' field and one of the following fields: \'map\', \'seq\', \'value\'"))
    }
}

struct DeserializeSerializableDeserializer<'a> {
    serializable_deserialize: &'a SerializableDeserialize,
}

impl<'a, 'de> DeserializeSeed<'de> for DeserializeSerializableDeserializer<'a> {
    type Value = Box<dyn Serializable>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.serializable_deserialize.deserialize(deserializer)
    }
}

struct ListDeserializer<'a> {
    registry: &'a SerializableRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for ListDeserializer<'a> {
    type Value = SerializableDynamicList;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ListVisitor {
            registry: self.registry,
        })
    }
}

struct ListVisitor<'a> {
    registry: &'a SerializableRegistry,
}

impl<'a, 'de> Visitor<'de> for ListVisitor<'a> {
    type Value = SerializableDynamicList;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("list value")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut list = SerializableDynamicList::default();
        while let Some(value) = seq.next_element_seed(SerializableDeserializer {
            registry: self.registry,
        })? {
            list.push_boxed(value);
        }
        Ok(list)
    }
}

struct ArrayDeserializer<'a> {
    registry: &'a SerializableRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for ArrayDeserializer<'a> {
    type Value = SerializableDynamicArray;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ArrayVisitor {
            registry: self.registry,
        })
    }
}

struct ArrayVisitor<'a> {
    registry: &'a SerializableRegistry,
}

impl<'a, 'de> Visitor<'de> for ArrayVisitor<'a> {
    type Value = SerializableDynamicArray;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("array value")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or_default());
        while let Some(value) = seq.next_element_seed(SerializableDeserializer {
            registry: self.registry,
        })? {
            vec.push(value);
        }

        Ok(SerializableDynamicArray::new(Box::from(vec)))
    }
}

struct MapDeserializer<'a> {
    registry: &'a SerializableRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for MapDeserializer<'a> {
    type Value = DynamicSerializableMap;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(MapVisitor {
            registry: self.registry,
        })
    }
}

struct MapVisitor<'a> {
    registry: &'a SerializableRegistry,
}

impl<'a, 'de> Visitor<'de> for MapVisitor<'a> {
    type Value = DynamicSerializableMap;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("map value")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut map = DynamicSerializableMap::default();
        while let Some(key) = seq.next_element_seed(SerializableDeserializer {
            registry: self.registry,
        })? {
            if let Some(value) = seq.next_element_seed(SerializableDeserializer {
                registry: self.registry,
            })? {
                map.insert_boxed(key, value);
            }
        }
        Ok(map)
    }
}

struct StructDeserializer<'a> {
    registry: &'a SerializableRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for StructDeserializer<'a> {
    type Value = SerializableDynamicStruct;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(StructVisitor {
            registry: self.registry,
        })
    }
}

struct StructVisitor<'a> {
    registry: &'a SerializableRegistry,
}

impl<'a, 'de> Visitor<'de> for StructVisitor<'a> {
    type Value = SerializableDynamicStruct;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct value")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut dynamic_struct = SerializableDynamicStruct::default();
        while let Some(key) = map.next_key::<String>()? {
            let value = map.next_value_seed(SerializableDeserializer {
                registry: self.registry,
            })?;
            dynamic_struct.insert_boxed(&key, value);
        }

        Ok(dynamic_struct)
    }
}

struct BoxDeserializer<'a> {
    registry: &'a SerializableRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for BoxDeserializer<'a> {
    type Value = Box<dyn Serializable>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(BoxVisitor {
            registry: self.registry,
        })
    }
}

struct BoxVisitor<'a> {
    registry: &'a SerializableRegistry,
}

impl<'a, 'de> Visitor<'de> for BoxVisitor<'a> {
    type Value = Box<dyn Serializable>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("box value")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let _key = map.next_key::<String>()?;
        let value = map.next_value_seed(SerializableDeserializer {
            registry: self.registry,
        })?;
        Ok(value)
    }
}

struct TupleStructDeserializer<'a> {
    registry: &'a SerializableRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for TupleStructDeserializer<'a> {
    type Value = SerializableDynamicTupleStruct;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(TupleStructVisitor {
            registry: self.registry,
        })
    }
}

struct TupleStructVisitor<'a> {
    registry: &'a SerializableRegistry,
}

impl<'a, 'de> Visitor<'de> for TupleStructVisitor<'a> {
    type Value = SerializableDynamicTupleStruct;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("tuple struct value")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut tuple_struct = SerializableDynamicTupleStruct::default();
        while let Some(value) = seq.next_element_seed(SerializableDeserializer {
            registry: self.registry,
        })? {
            tuple_struct.insert_boxed(value);
        }
        Ok(tuple_struct)
    }
}

struct TupleDeserializer<'a> {
    registry: &'a SerializableRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for TupleDeserializer<'a> {
    type Value = SerializableDynamicTuple;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(TupleVisitor {
            registry: self.registry,
        })
    }
}

struct TupleVisitor<'a> {
    registry: &'a SerializableRegistry,
}

impl<'a, 'de> Visitor<'de> for TupleVisitor<'a> {
    type Value = SerializableDynamicTuple;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("tuple value")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut tuple = SerializableDynamicTuple::default();
        while let Some(value) = seq.next_element_seed(SerializableDeserializer {
            registry: self.registry,
        })? {
            tuple.insert_boxed(value);
        }
        Ok(tuple)
    }
}
