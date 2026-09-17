use std::{
    fs,
    path::{Path, PathBuf},
};

use inox_binarizer::ShaderCompiler;
use inox_graphics::{wavefront_production_shader_files, WAVEFRONT_PRODUCTION_PIPELINES};
use inox_platform::PLATFORM_TYPE_WEB;

fn pipeline_shader_from_source(root: &Path, pipeline: &str) -> String {
    let file_name = PathBuf::from(pipeline)
        .file_name()
        .expect("pipeline descriptor must include a file name")
        .to_owned();
    let source = fs::read_to_string(root.join("../../pipelines").join(file_name))
        .expect("active pipeline descriptor must exist");
    let shader = source
        .split("\"shader\"")
        .nth(1)
        .and_then(|value| value.split('"').nth(1))
        .expect("pipeline descriptor must reference a shader");
    shader
        .rsplit('/')
        .next()
        .unwrap()
        .strip_suffix(".shader")
        .expect("pipeline shader must use the source .shader extension")
        .to_owned()
        + ".wgsl"
}

#[test]
fn active_wavefront_pipeline_descriptors_fit_the_portable_storage_budget() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data_raw/shaders/wgsl");
    for descriptor in WAVEFRONT_PRODUCTION_PIPELINES {
        assert_eq!(
            descriptor.shader,
            pipeline_shader_from_source(&root, descriptor.pipeline),
            "pipeline file mapping diverged for {:?}",
            descriptor.pipeline
        );
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
fn active_wavefront_pass_bindings_do_not_chain_buffer_writes() {
    const PASS_FILES: &[&str] = &[
        "compute_wavefront_ao_resolve.rs",
        "compute_wavefront_composite.rs",
        "compute_wavefront_denoise.rs",
        "compute_wavefront_direct_candidates.rs",
        "compute_wavefront_dispatch.rs",
        "compute_wavefront_next_bounce.rs",
        "compute_wavefront_primary.rs",
        "compute_wavefront_shade.rs",
        "compute_wavefront_surface.rs",
        "compute_wavefront_transparency.rs",
        "compute_wavefront_traversal.rs",
        "compute_wavefront_upscale.rs",
        "compute_wavefront_visibility.rs",
    ];
    let passes_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/passes");
    for file in PASS_FILES {
        let source = fs::read_to_string(passes_dir.join(file)).unwrap();
        assert!(
            !source.contains(")\n            .add_buffer("),
            "wavefront pass chains add_buffer calls: {file}"
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
