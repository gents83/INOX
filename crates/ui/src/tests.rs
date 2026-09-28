use super::systems::ui_system::logical_screen_size;

#[test]
fn logical_screen_size_converts_physical_resize_once() {
    assert_eq!(logical_screen_size(1920, 1080, 2.0), [960.0, 540.0]);
}

#[test]
fn logical_screen_size_clamps_invalid_scale() {
    assert_eq!(logical_screen_size(800, 600, 0.0), [800.0, 600.0]);
    assert_eq!(logical_screen_size(800, 600, f32::NAN), [800.0, 600.0]);
}

#[test]
fn ui_shader_converts_tessellated_physical_vertices_once() {
    let source = include_str!("../../../data_raw/shaders/wgsl/ui.wgsl");
    assert!(source.contains("v_in.position.x * ui_scale / constant_data.screen_width"));
    assert!(source.contains("v_in.position.y * ui_scale / constant_data.screen_height"));
}

#[test]
fn ui_vertex_buffers_use_stable_gpu_layouts() {
    let source = include_str!("systems/ui_pass.rs");
    let vertex = source.find("pub struct UIVertex").unwrap();
    let instance = source.find("pub struct UIInstance").unwrap();
    assert!(source[..vertex].contains("#[repr(C)]"));
    assert!(source[..instance].contains("#[repr(C)]"));
}

#[test]
fn ui_meshes_wait_for_managed_texture_uploads() {
    let source = include_str!("systems/ui_system.rs");
    assert!(source.contains("texture.get().texture_index() < 0"));
}

#[test]
fn ui_pass_refreshes_texture_atlas_bindings_after_font_upload() {
    let source = include_str!("systems/ui_pass.rs");
    assert!(source.matches("add_material_textures").count() >= 2);
    assert!(source.contains("textures_atlas().is_empty()"));
}
