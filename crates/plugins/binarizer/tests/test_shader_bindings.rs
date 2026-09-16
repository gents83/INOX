#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use inox_binarizer::{ShaderCompiler, MAX_PORTABLE_STORAGE_BUFFERS};
use inox_platform::PLATFORM_TYPE_WEB;

#[test]
fn accepts_eight_storage_buffers_after_include_expansion() {
    let mut source = String::from("@compute\n");
    for binding in 0..MAX_PORTABLE_STORAGE_BUFFERS {
        source.push_str(&format!(
            "@group(0) @binding({binding}) var<storage, read> buffer_{binding}: array<u32>;\n"
        ));
    }
    let report = ShaderCompiler::<{ PLATFORM_TYPE_WEB }>::extract_storage_bindings(
        &source,
        Path::new("eight.wgsl"),
    )
    .unwrap();
    assert_eq!(report.count(), MAX_PORTABLE_STORAGE_BUFFERS);
}

#[test]
fn rejects_ninth_storage_buffer_with_path_stage_and_count() {
    let source = (0..=MAX_PORTABLE_STORAGE_BUFFERS)
        .map(|binding| {
            format!(
                "@group(0) @binding({binding}) var<storage, read> buffer_{binding}: array<u32>;"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let error = ShaderCompiler::<{ PLATFORM_TYPE_WEB }>::extract_storage_bindings(
        &source,
        Path::new("production.wgsl"),
    )
    .unwrap_err();
    assert!(error.contains("production.wgsl"));
    assert!(error.contains("storage bindings"));
    assert!(error.contains("9"));
}

#[test]
fn rejects_duplicate_and_missing_storage_binding_attributes() {
    let duplicate = "@group(0) @binding(1) var<storage, read> a: array<u32>;\n@group(0) @binding(1) var<storage, read> b: array<u32>;";
    let duplicate_error = ShaderCompiler::<{ PLATFORM_TYPE_WEB }>::extract_storage_bindings(
        duplicate,
        Path::new("duplicate.wgsl"),
    )
    .unwrap_err();
    assert!(duplicate_error.contains("duplicate storage binding"));

    let missing = "var<storage, read> missing: array<u32>;";
    let missing_error = ShaderCompiler::<{ PLATFORM_TYPE_WEB }>::extract_storage_bindings(
        missing,
        Path::new("missing.wgsl"),
    )
    .unwrap_err();
    assert!(missing_error.contains("missing @group"));
}

#[test]
fn validates_all_raw_pipeline_shader_references() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../data_raw");
    ShaderCompiler::<{ PLATFORM_TYPE_WEB }>::validate_pipeline_descriptors(&root).unwrap();
}
