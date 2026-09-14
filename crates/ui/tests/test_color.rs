use inox_math::{Vector3, Vector4};
use inox_ui::{hex_to_rgb, hex_to_rgba, rgba_to_u32, Color};

fn assert_close(actual: Vector3, expected: Vector3) {
    assert!((actual.x - expected.x).abs() < f32::EPSILON);
    assert!((actual.y - expected.y).abs() < f32::EPSILON);
    assert!((actual.z - expected.z).abs() < f32::EPSILON);
}

#[test]
fn parses_short_and_long_hex_colors() {
    assert_close(
        hex_to_rgb("#336699"),
        Vector3::new(0x33 as f32 / 255., 0x66 as f32 / 255., 0x99 as f32 / 255.),
    );
    assert_close(hex_to_rgb("fff"), Vector3::new(1., 1., 1.));
    assert_eq!(hex_to_rgba("#000000"), Vector4::new(0., 0., 0., 1.));
}

#[test]
fn clamps_color_arithmetic_and_packs_rgba() {
    let mut color = Vector4::new(0.8, 0.2, 0.0, 1.0);
    assert_eq!(
        color.add_color(Vector4::new(0.5, 0.5, 0.5, 0.0)),
        Vector4::new(1.0, 0.7, 0.5, 1.0)
    );
    assert_eq!(
        color.remove_color(Vector4::new(0.2, 0.8, 0.8, 0.0)),
        Vector4::new(0.8, 0.0, 0.0, 1.0)
    );
    assert_eq!(rgba_to_u32(1., 2., 3., 4.), 0x01020304);
}
