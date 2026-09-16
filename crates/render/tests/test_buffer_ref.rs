use inox_render::validate_buffer_overwrite_range;

#[test]
fn overwrite_range_uses_logical_input_size_without_shrinking_capacity() {
    assert_eq!(validate_buffer_overwrite_range(256, 64), Ok(0..64));
    assert_eq!(validate_buffer_overwrite_range(256, 256), Ok(0..256));
}

#[test]
fn overwrite_range_rejects_input_larger_than_capacity() {
    assert!(validate_buffer_overwrite_range(256, 257).is_err());
}
