use std::path::PathBuf;

use inox_binarizer::ShaderCompiler;
use inox_graphics::{wavefront_production_shader_files, WAVEFRONT_PRODUCTION_PIPELINES};
use inox_platform::PLATFORM_TYPE_WEB;

#[test]
fn active_wavefront_pipeline_descriptors_fit_the_portable_storage_budget() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data_raw/shaders/wgsl");
    for descriptor in WAVEFRONT_PRODUCTION_PIPELINES {
        let path = root.join(descriptor.shader);
        let source = std::fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!(
                "unable to read shader {:?} referenced by pipeline {:?}: {error}",
                descriptor.shader, descriptor.pipeline
            )
        });
        assert!(
            source.matches("var<storage").count() <= 8,
            "active wavefront shader exceeds portable storage budget: {path:?}"
        );
        assert!(
            ShaderCompiler::<{ PLATFORM_TYPE_WEB }>::validate_wgsl_file(&path),
            "active wavefront pipeline descriptor failed include-expanded binding validation: {:?}",
            descriptor.pipeline
        );
    }
}

#[test]
fn wavefront_shader_inventory_matches_pipeline_descriptors_bidirectionally() {
    let descriptor_shaders = WAVEFRONT_PRODUCTION_PIPELINES
        .iter()
        .map(|descriptor| descriptor.shader)
        .collect::<std::collections::BTreeSet<_>>();
    let inventory_shaders = wavefront_production_shader_files()
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    assert!(!descriptor_shaders.is_empty());
    assert_eq!(descriptor_shaders, inventory_shaders);

    let pipelines = WAVEFRONT_PRODUCTION_PIPELINES
        .iter()
        .map(|descriptor| descriptor.pipeline)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(pipelines.len(), WAVEFRONT_PRODUCTION_PIPELINES.len());
}
