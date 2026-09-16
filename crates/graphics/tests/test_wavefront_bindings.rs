use std::path::PathBuf;

use inox_binarizer::ShaderCompiler;
use inox_graphics::WAVEFRONT_PRODUCTION_SHADER_FILES;
use inox_platform::PLATFORM_TYPE_WEB;

#[test]
fn active_wavefront_pipeline_shaders_fit_the_portable_storage_budget() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data_raw/shaders/wgsl");
    for shader in WAVEFRONT_PRODUCTION_SHADER_FILES {
        let path = root.join(shader);
        let source = std::fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("unable to read active wavefront shader {path:?}: {error}")
        });
        assert!(
            source.matches("var<storage").count() <= 8,
            "active wavefront shader exceeds portable storage budget: {path:?}"
        );
        assert!(
            ShaderCompiler::<{ PLATFORM_TYPE_WEB }>::validate_wgsl_file(&path),
            "active wavefront shader failed generated binding validation: {path:?}"
        );
    }
}

#[test]
fn active_wavefront_shader_inventory_is_unique_and_nonempty() {
    assert!(!WAVEFRONT_PRODUCTION_SHADER_FILES.is_empty());
    let mut shaders = WAVEFRONT_PRODUCTION_SHADER_FILES.to_vec();
    shaders.sort_unstable();
    shaders.dedup();
    assert_eq!(shaders.len(), WAVEFRONT_PRODUCTION_SHADER_FILES.len());
}
