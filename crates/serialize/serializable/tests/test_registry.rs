use inox_serializable::{DeserializeFn, Registry};

fn deserialize_u32(
    _deserializer: &mut dyn inox_serializable::erased_serde::Deserializer,
) -> inox_serializable::erased_serde::Result<Box<u32>> {
    unreachable!("registration test does not deserialize values")
}

#[test]
fn registry_tracks_sorted_names_and_removes_types() {
    let mut registry = Registry::<u32>::default();
    let function: DeserializeFn<u32> = deserialize_u32;

    registry.register_type("zulu", function);
    registry.register_type("alpha", function);
    assert_eq!(registry.names, vec!["alpha", "zulu"]);
    assert!(registry.map.contains_key("alpha"));

    assert!(!registry.unregister_type("alpha"));
    assert!(registry.unregister_type("zulu"));
    assert!(registry.names.is_empty());
    assert!(registry.map.is_empty());
}
