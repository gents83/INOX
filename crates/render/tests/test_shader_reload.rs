use std::path::{Path, PathBuf};

use std::sync::Arc;

use inox_messenger::MessageHub;
use inox_render::{ComputePipeline, RenderPipeline, Shader, ShaderData};
use inox_resources::{DataTypeResource, ResourceTrait, SharedData};
use inox_uid::generate_random_uid;

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

fn reload_context() -> (Arc<SharedData>, Arc<MessageHub>) {
    (
        Arc::new(SharedData::default()),
        Arc::new(MessageHub::default()),
    )
}

#[test]
fn concrete_shader_retains_active_state_across_failed_candidate() {
    let (shared_data, message_hub) = reload_context();
    let mut active = Shader::new(generate_random_uid(), &shared_data, &message_hub);
    let candidate = Shader::create_from_data(
        &shared_data,
        &message_hub,
        generate_random_uid(),
        &ShaderData {
            spirv_code: Vec::new(),
            wgsl_code: "invalid wgsl".to_string(),
        },
    );

    active.reload_state_mut().activate();
    active.replace_from(&candidate);
    let error = ReloadError::new(
        Path::new("data/shaders/lighting.wgsl"),
        "shader",
        "expected expression",
        true,
    );
    active.record_reload_failure(error.clone());

    assert!(active.reload_state().active());
    assert!(active.reload_state().pending());
    assert_eq!(active.last_reload_error(), Some(&error));

    active.replace_from(&candidate);
    active.reload_state_mut().commit();
    assert!(active.reload_state().active());
    assert!(!active.reload_state().pending());
    assert!(active.reload_state().last_error().is_none());
}

#[test]
fn concrete_compute_pipeline_retains_active_state_then_commits() {
    let (shared_data, message_hub) = reload_context();
    let mut active = ComputePipeline::new(generate_random_uid(), &shared_data, &message_hub);
    let candidate = ComputePipeline::new(generate_random_uid(), &shared_data, &message_hub);

    active.reload_state_mut().activate();
    active.replace_from(&candidate);
    let error = ReloadError::new(
        Path::new("data/pipelines/lighting.compute_pipeline"),
        "compute pipeline",
        "pipeline layout mismatch",
        true,
    );
    active.record_reload_failure(error.clone());

    assert!(active.reload_state().active());
    assert!(active.reload_state().pending());
    assert_eq!(active.last_reload_error(), Some(&error));

    active.replace_from(&candidate);
    active.reload_state_mut().commit();
    assert!(active.reload_state().active());
    assert!(!active.reload_state().pending());
}

#[test]
fn concrete_render_pipeline_retains_active_state_then_commits() {
    let (shared_data, message_hub) = reload_context();
    let mut active = RenderPipeline::new(generate_random_uid(), &shared_data, &message_hub);
    let candidate = RenderPipeline::new(generate_random_uid(), &shared_data, &message_hub);

    active.reload_state_mut().activate();
    active.replace_from(&candidate);
    let error = ReloadError::new(
        Path::new("data/pipelines/lighting.render_pipeline"),
        "render pipeline",
        "fragment target mismatch",
        true,
    );
    active.record_reload_failure(error.clone());

    assert!(active.reload_state().active());
    assert!(active.reload_state().pending());
    assert_eq!(active.last_reload_error(), Some(&error));

    active.replace_from(&candidate);
    active.reload_state_mut().commit();
    assert!(active.reload_state().active());
    assert!(!active.reload_state().pending());
}
