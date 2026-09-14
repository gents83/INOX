use inox_uid::{
    generate_static_uid_from_string, generate_uid_from_string, generate_uid_from_type, INVALID_UID,
};

#[test]
fn string_uid_generation_is_deterministic() {
    assert_eq!(
        generate_uid_from_string("player"),
        generate_uid_from_string("player")
    );
    assert_ne!(
        generate_uid_from_string("player"),
        generate_uid_from_string("enemy")
    );
}

#[test]
fn static_uid_is_non_nil_and_type_uid_is_stable() {
    let static_uid = generate_static_uid_from_string("player");
    assert_ne!(static_uid, INVALID_UID);
    assert_eq!(
        generate_uid_from_type::<u32>(),
        generate_uid_from_type::<u32>()
    );
    assert_ne!(
        generate_uid_from_type::<u32>(),
        generate_uid_from_type::<u64>()
    );
}
