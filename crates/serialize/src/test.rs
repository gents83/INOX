use serde::de::DeserializeSeed;
use serde_json::{de::StrRead, to_string_pretty};
use std::collections::HashMap;

use super::*;
use std::any::Any;

#[serializable_trait]
pub trait TestBaseTrait: Serializable + Send + Sync + 'static + Any {
    fn value(&self) -> u32;
    fn clone_trait(&self) -> Box<dyn TestBaseTrait>;
}

#[derive(Default, Serializable, Eq, PartialEq, Debug, Clone)]
#[serializable(TestBaseTrait)]
struct Foo {
    x: u32,
}

impl TestBaseTrait for Foo {
    fn value(&self) -> u32 {
        self.x
    }
    fn clone_trait(&self) -> Box<dyn TestBaseTrait> {
        Box::new(self.clone())
    }
}

#[derive(Default, Serializable, PartialEq, Debug, Clone)]
#[serializable(TestBaseTrait)]
struct Bar {
    y: u32,
    z: f32,
}

impl TestBaseTrait for Bar {
    fn value(&self) -> u32 {
        self.y
    }
    fn clone_trait(&self) -> Box<dyn TestBaseTrait> {
        Box::new(self.clone())
    }
}

#[derive(Serializable, Clone)]
struct MyMap {
    map: HashMap<String, Box<dyn TestBaseTrait>>,
    foo: Box<dyn TestBaseTrait>,
    bar: Box<dyn TestBaseTrait>,
}

impl Default for MyMap {
    fn default() -> Self {
        MyMap {
            map: HashMap::new(),
            foo: Box::new(Foo::default()),
            bar: Box::new(Bar::default()),
        }
    }
}

#[derive(Serializable, Clone, Debug)]
enum SimpleEnum {
    A,
    B,
    C,
}

#[derive(Serializable, Clone, Debug)]
enum ComplexEnum {
    Empty,
    FooValue(Foo),
    BarValue(Bar),
}

#[allow(dead_code)]
fn test_trait() {
    let mut registry = SerializableRegistry::default();
    registry.register_type::<f32>();
    registry.register_type::<u32>();
    registry.register_type::<String>();
    registry.register_trait::<dyn TestBaseTrait>();
    registry.register_type_with_trait::<dyn TestBaseTrait, Foo>();
    registry.register_type_with_trait::<dyn TestBaseTrait, Bar>();
    registry.register_type::<MyMap>();

    //Test trait casting
    let b = Box::new(Foo { x: 1 });
    assert!(b.value() == 1);

    let foo = Foo { x: 12 };
    let bar = Bar { y: 9, z: 19.83 };

    let my_map = MyMap {
        foo: foo.clone_trait(),
        bar: bar.clone_trait(),
        map: {
            let mut map = HashMap::new();
            map.insert("foo".to_string(), foo.clone_trait());
            map.insert("bar".to_string(), bar.clone_trait());
            map
        },
    };
    let serializer = SerializableSerializer::new(&my_map, &registry);
    let serialized = to_string_pretty(&serializer).unwrap();

    let mut json_deserializer = serde_json::Deserializer::new(StrRead::new(serialized.as_str()));
    let deserializer = SerializableDeserializer::new(&registry);

    let value = deserializer.deserialize(&mut json_deserializer).unwrap();

    println!("value type_name: {:?}", value.type_name());
    println!("value ref type_name: {:?}", value.as_ref().type_name());

    let deserialized = registry
        .get_with_fullname(value.as_ref().type_name().as_str())
        .unwrap();
    println!(
        "deserialized id: {:?} with type {:?}",
        deserialized.type_id(),
        deserialized.name()
    );

    assert!(deserialized.type_id() == my_map.type_id());

    let mut m = MyMap::default();
    println!("foo value: {:?}", foo.value());
    println!("bar value: {:?}", bar.value());

    println!(
        "m.foo default value: {:?}",
        m.foo.any().downcast_ref::<Foo>().unwrap().value()
    );
    println!(
        "m.bar default value: {:?}",
        m.bar.any().downcast_ref::<Bar>().unwrap().value()
    );
    assert!(m.foo.any().downcast_ref::<Foo>().unwrap().value() == 0);
    assert!(m.bar.any().downcast_ref::<Bar>().unwrap().value() == 0);
    assert!(m.map.is_empty());

    println!("m type_name: {:?}", m.type_name());

    m.set(&value, &registry);

    println!(
        "m.foo Type: {:?} value: {:?}",
        m.foo.type_name(),
        m.foo.any().downcast_ref::<Foo>().unwrap().value()
    );
    println!(
        "m.bar Type: {:?} value: {:?}",
        m.bar.type_name(),
        m.bar.any().downcast_ref::<Bar>().unwrap().value()
    );
    assert!(m.foo.any().downcast_ref::<Foo>().unwrap().value() == foo.value());
    assert!(m.bar.any().downcast_ref::<Bar>().unwrap().value() == bar.value());
    assert!(m.map.len() == 2);
    for (s, item) in m.map.iter() {
        println!("String {} Type: {:?}", s, item.type_name());
        match s.as_str() {
            "foo" => {
                assert!(item.any().downcast_ref::<Foo>().unwrap().value() == foo.value());
            }
            "bar" => {
                assert!(item.any().downcast_ref::<Bar>().unwrap().value() == bar.value());
            }
            _ => panic!("unexpected type"),
        }
    }

    assert!(foo.fields_count() == 1);
    assert!(foo.name_at(0).unwrap() == "x");
    assert!(bar.fields_count() == 2);
    assert!(bar.name_at(0).unwrap() == "y");
    assert!(bar.name_at(1).unwrap() == "z");

    for (i, f) in bar.iter_fields().enumerate() {
        println!("field {}: {:?}", i, f.type_name(),);
    }
}

#[allow(dead_code)]
fn test_enum() {
    let mut registry = SerializableRegistry::default();
    registry.register_type::<f32>();
    registry.register_type::<u32>();
    registry.register_type::<usize>();
    registry.register_type::<String>();
    registry.register_trait::<dyn TestBaseTrait>();
    registry.register_type_with_trait::<dyn TestBaseTrait, Foo>();
    registry.register_type_with_trait::<dyn TestBaseTrait, Bar>();
    registry.register_type::<SimpleEnum>();
    registry.register_type::<ComplexEnum>();

    let simple = SimpleEnum::B;
    assert!(!simple.is_equal(&SimpleEnum::A).unwrap());
    assert!(simple.is_equal(&SimpleEnum::B).unwrap());
    assert!(!simple.is_equal(&SimpleEnum::C).unwrap());

    let serializer = SerializableSerializer::new(&simple, &registry);
    let serialized = to_string_pretty(&serializer).unwrap();

    println!("simple enum:\n{}", serialized);

    let mut json_deserializer = serde_json::Deserializer::new(StrRead::new(serialized.as_str()));
    let deserializer = SerializableDeserializer::new(&registry);

    let value = deserializer.deserialize(&mut json_deserializer).unwrap();

    println!("value type_name: {:?}", value.type_name());
    println!("value ref type_name: {:?}", value.as_ref().type_name());

    let mut v = SimpleEnum::A;
    println!("default simple enum value: {:?}", v);

    v.set(value.as_ref(), &registry);

    println!("deserialized simple enum value: {:?}", v);

    let mut complex = ComplexEnum::Empty;
    assert!(complex.is_equal(&ComplexEnum::Empty).unwrap());
    complex = ComplexEnum::FooValue(Foo { x: 1 });
    assert!(!complex.is_equal(&ComplexEnum::Empty).unwrap());
    assert!(!complex
        .is_equal(&ComplexEnum::BarValue(Bar { y: 2, z: 3.4 }))
        .unwrap());

    let serializer = SerializableSerializer::new(&complex, &registry);
    let serialized = to_string_pretty(&serializer).unwrap();

    println!("complex enum:\n{}", serialized);
}

#[test]
fn serialization_tests() {
    test_trait();

    test_enum();
}
