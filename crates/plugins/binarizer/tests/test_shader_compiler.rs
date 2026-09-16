#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use inox_binarizer::ShaderCompiler;
use inox_platform::{PLATFORM_TYPE_PC, PLATFORM_TYPE_WEB};

fn pc_defs() -> Vec<String> {
    inox_render::platform::shader_preprocessor_defs::<{ PLATFORM_TYPE_PC }>()
}

#[test]
fn desktop_profile_uses_primitive_index_fallback_for_portable_adapters() {
    let defs = pc_defs();
    assert!(defs.contains(&"NO_FEATURES_PRIMITIVE_INDEX".to_string()));
    assert!(!defs.contains(&"FEATURES_PRIMITIVE_INDEX".to_string()));
}

fn web_defs() -> Vec<String> {
    inox_render::platform::shader_preprocessor_defs::<{ PLATFORM_TYPE_WEB }>()
}

#[test]
fn adds_binding_array_extension_before_shader_items() {
    let source = "const VALUE: u32 = 1u;\nvar textures: binding_array<texture_2d<f32>, 4>;\n";
    let result = ShaderCompiler::<{ PLATFORM_TYPE_PC }>::add_required_wgsl_extensions(
        source.to_string(),
        &pc_defs(),
    );

    assert!(result.starts_with("enable wgpu_binding_array;\n"));
    assert_eq!(result.matches("enable wgpu_binding_array;").count(), 1);
}

#[test]
fn adds_primitive_index_extension_when_feature_is_available() {
    let source = "@fragment\nfn main(@builtin(primitive_index) index: u32) {}\n";
    let result = ShaderCompiler::<{ PLATFORM_TYPE_PC }>::add_required_wgsl_extensions(
        source.to_string(),
        &["FEATURES_PRIMITIVE_INDEX".to_string()],
    );

    assert!(result.starts_with("enable primitive_index;\n"));
}

#[test]
fn does_not_enable_primitive_index_for_web_shaders() {
    let source = "@fragment\nfn main(@builtin(primitive_index) index: u32) {}\n";
    let result = ShaderCompiler::<{ PLATFORM_TYPE_WEB }>::add_required_wgsl_extensions(
        source.to_string(),
        &web_defs(),
    );

    assert_eq!(result, source);
}

#[test]
fn does_not_duplicate_existing_binding_array_extension() {
    let source = "enable wgpu_binding_array;\nvar textures: binding_array<texture_2d<f32>, 4>;\n";
    let result = ShaderCompiler::<{ PLATFORM_TYPE_PC }>::add_required_wgsl_extensions(
        source.to_string(),
        &pc_defs(),
    );

    assert_eq!(result, source);
}

#[test]
fn leaves_shader_without_binding_array_unchanged() {
    let source = "const VALUE: u32 = 1u;\n";
    let result = ShaderCompiler::<{ PLATFORM_TYPE_PC }>::add_required_wgsl_extensions(
        source.to_string(),
        &pc_defs(),
    );

    assert_eq!(result, source);
}

#[test]
fn wavefront_stages_validate_with_the_web_shader_profile() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data_raw/shaders/wgsl");
    for shader in [
        "compute_wavefront_primary.wgsl",
        "compute_wavefront_dispatch.wgsl",
        "compute_wavefront_traversal.wgsl",
        "compute_wavefront_shade.wgsl",
        "compute_wavefront_reset.wgsl",
        "compute_wavefront_denoise_temporal.wgsl",
        "compute_wavefront_denoise_bilateral.wgsl",
        "compute_wavefront_direct_candidates.wgsl",
        "compute_wavefront_composite.wgsl",
        "compute_wavefront_ao_resolve.wgsl",
        "compute_wavefront_surface.wgsl",
        "compute_wavefront_visibility.wgsl",
        "compute_wavefront_upscale.wgsl",
        "compute_wavefront_transparency.wgsl",
        "compute_wavefront_transparency_resolve.wgsl",
        "compute_wavefront_next_bounce.wgsl",
    ] {
        let path = root.join(shader);
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(
            source.matches("var<storage").count() <= 8,
            "portable shader exceeds storage binding budget: {path:?}"
        );
        assert!(
            ShaderCompiler::<{ PLATFORM_TYPE_WEB }>::validate_wgsl_file(&path),
            "failed to validate {path:?}"
        );
    }
    let visibility = root.join("visibility.wgsl");
    assert!(ShaderCompiler::<{ PLATFORM_TYPE_PC }>::validate_wgsl_file(
        &visibility
    ));
}

#[test]
fn detects_binding_array_with_whitespace_before_the_angle_bracket() {
    let source = "var textures: binding_array <texture_2d<f32>, 4>;\n";
    let result = ShaderCompiler::<{ PLATFORM_TYPE_PC }>::add_required_wgsl_extensions(
        source.to_string(),
        &pc_defs(),
    );

    assert!(result.starts_with("enable wgpu_binding_array;\n"));
}

#[test]
fn detects_primitive_index_with_whitespace_inside_builtin_attribute() {
    let source = "@fragment\nfn main(@builtin( primitive_index ) index: u32) {}\n";
    let result = ShaderCompiler::<{ PLATFORM_TYPE_PC }>::add_required_wgsl_extensions(
        source.to_string(),
        &["FEATURES_PRIMITIVE_INDEX".to_string()],
    );

    assert!(result.starts_with("enable primitive_index;\n"));
}

#[test]
fn does_not_duplicate_extensions_with_trailing_comments() {
    let source = "enable primitive_index; // already enabled\n@fragment\nfn main(@builtin(primitive_index) index: u32) {}\n";
    let result = ShaderCompiler::<{ PLATFORM_TYPE_PC }>::add_required_wgsl_extensions(
        source.to_string(),
        &["FEATURES_PRIMITIVE_INDEX".to_string()],
    );

    assert_eq!(result.matches("enable primitive_index;").count(), 1);
}

#[test]
fn leaves_binding_array_shader_unchanged_when_feature_is_unavailable() {
    let source = "var textures: binding_array<texture_2d<f32>, 4>;\n";
    let result = ShaderCompiler::<{ PLATFORM_TYPE_PC }>::add_required_wgsl_extensions(
        source.to_string(),
        &[],
    );

    assert_eq!(result, source);
}
