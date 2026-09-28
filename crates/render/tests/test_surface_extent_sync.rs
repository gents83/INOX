#[test]
fn acquired_surface_extent_is_checked_before_frame_encoding() {
    let source = include_str!("../src/common/render_context.rs");
    let acquisition = source
        .find("pub fn obtain_surface_texture")
        .expect("surface acquisition path must exist");
    let acquisition = &source[acquisition..];
    assert!(acquisition.contains("surface_texture.texture.width()"));
    assert!(acquisition.contains("surface_texture.texture.height()"));
    assert!(acquisition.contains("set_surface_size(surface_width, surface_height)"));
}
