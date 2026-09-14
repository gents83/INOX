#![cfg(not(target_arch = "wasm32"))]

use inox_binarizer::ShaderCompiler;
use inox_platform::{PLATFORM_TYPE_PC, PLATFORM_TYPE_WEB};

fn pc_defs() -> Vec<String> {
    inox_render::platform::shader_preprocessor_defs::<{ PLATFORM_TYPE_PC }>()
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
        &pc_defs(),
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
fn leaves_binding_array_shader_unchanged_when_feature_is_unavailable() {
    let source = "var textures: binding_array<texture_2d<f32>, 4>;\n";
    let result = ShaderCompiler::<{ PLATFORM_TYPE_PC }>::add_required_wgsl_extensions(
        source.to_string(),
        &[],
    );

    assert_eq!(result, source);
}
