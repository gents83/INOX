use std::path::{Path, PathBuf};

use inox_render::resources::shader::{is_shader, ReloadError};

#[test]
fn failed_reload_reports_location_and_retains_active_resource() {
    let error = ReloadError {
        path: PathBuf::from("data/shaders/lighting.wgsl"),
        stage: "fragment".to_string(),
        source_location: Some("line 12, column 4".to_string()),
        diagnostic: "expected expression".to_string(),
        previous_resource_active: true,
    };

    assert_eq!(error.path, Path::new("data/shaders/lighting.wgsl"));
    assert_eq!(error.stage, "fragment");
    assert_eq!(error.source_location.as_deref(), Some("line 12, column 4"));
    assert_eq!(error.diagnostic, "expected expression");
    assert!(error.previous_resource_active);
}

#[test]
fn reload_path_filter_rejects_paths_without_shader_extensions() {
    assert!(is_shader(Path::new("data/shaders/lighting.wgsl")));
    assert!(is_shader(Path::new("data/shaders/lighting.spv")));
    assert!(!is_shader(Path::new("data/shaders/lighting")));
    assert!(!is_shader(Path::new(
        "data/pipelines/lighting.compute_pipeline"
    )));
}
