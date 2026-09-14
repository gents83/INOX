use inox_math::{
    pack_4_f32_to_snorm, pack_4_f32_to_unorm, unpack_snorm_to_4_f32, unpack_unorm_to_4_f32, Vector4,
};

fn assert_vector_close(actual: Vector4, expected: Vector4, epsilon: f32) {
    assert!((actual.x - expected.x).abs() <= epsilon);
    assert!((actual.y - expected.y).abs() <= epsilon);
    assert!((actual.z - expected.z).abs() <= epsilon);
    assert!((actual.w - expected.w).abs() <= epsilon);
}

#[test]
fn unorm_packing_round_trips_boundaries() {
    let value = Vector4::new(0.0, 0.25, 0.5, 1.0);
    let packed = pack_4_f32_to_unorm(value);
    let decoded = unpack_unorm_to_4_f32(packed);

    assert_vector_close(decoded, value, 1.0 / 255.0);
}

#[test]
fn snorm_packing_round_trips_signed_values() {
    let value = Vector4::new(-1.0, -0.25, 0.5, 1.0);
    let packed = pack_4_f32_to_snorm(value);
    let decoded = unpack_snorm_to_4_f32(packed);

    assert_vector_close(decoded, value, 1.0 / 127.0);
}
