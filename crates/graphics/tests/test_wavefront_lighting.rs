use std::fs;
use std::path::PathBuf;

#[test]
fn direct_candidate_shader_uses_alias_pdf_and_empty_fallback() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data_raw/shaders/wgsl/compute_wavefront_direct_candidates.wgsl");
    let source = fs::read_to_string(path).unwrap();
    assert!(source.contains("sample_light_alias"));
    assert!(source.contains("proposal_pdf = max(bitcast<f32>(sample.y), 0.000001)"));
    assert!(source.contains("proposal_pdf = 1.0 / f32(light_count)"));
    assert!(source.contains("reservoir.confidence"));
    assert!(source.contains("jacobian"));
}

#[test]
fn direct_candidate_shader_accumulates_visible_emission_without_a_light() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data_raw/shaders/wgsl/compute_wavefront_direct_candidates.wgsl");
    let source = fs::read_to_string(path).unwrap();
    assert!(source.contains("var primary_emission = vec3<f32>(0.0)"));
    assert!(source.contains("primary_emission = max(material_emission"));
    assert!(source.contains("estimate = estimate * base_color + primary_emission"));
}

#[test]
fn visibility_uses_the_transform_z_scale_component() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data_raw/shaders/wgsl/visibility.wgsl");
    let source = fs::read_to_string(path).unwrap();
    assert!(source.contains(
        "vec3<f32>(transform.position_scale_x.w, transform.bb_min_scale_y.w, transform.bb_max_scale_z.w)"
    ));
    let visibility_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data_raw/shaders/wgsl/compute_wavefront_visibility.wgsl");
    let visibility_source = fs::read_to_string(visibility_path).unwrap();
    assert!(visibility_source.contains("emissive * (1.0 - visibility)"));
}

#[test]
fn visibility_pipeline_writes_integer_hits_and_culling_keeps_single_lod_meshlets() {
    let pipeline = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data_raw/pipelines/VisibilityBuffer.render_pipeline");
    let pipeline_source = fs::read_to_string(pipeline).unwrap();
    assert!(pipeline_source.contains("\"src_color_blend_factor\": \"Zero\""));
    assert!(pipeline_source.contains("\"dst_color_blend_factor\": \"Zero\""));
    assert!(pipeline_source.contains("\"src_alpha_blend_factor\": \"Zero\""));
    assert!(pipeline_source.contains("\"dst_alpha_blend_factor\": \"Zero\""));

    let culling = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data_raw/shaders/wgsl/compute_culling.wgsl");
    let culling_source = fs::read_to_string(culling).unwrap();
    assert!(culling_source.contains("meshlet.parent_error <= MATH_EPSILON"));
    assert!(culling_source.contains("depth <= 0.0 || depth != depth"));

    let finalize =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data_raw/shaders/wgsl/finalize.wgsl");
    let finalize_source = fs::read_to_string(finalize).unwrap();
    assert!(finalize_source.contains("final_accumulator[pixel_index].history_length > 0u"));
    assert!(finalize_source
        .contains("final_accumulator[pixel_index].flags == constant_data.history_epoch"));
}

#[test]
fn direct_candidate_shader_keeps_the_portable_storage_budget() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data_raw/shaders/wgsl/compute_wavefront_direct_candidates.wgsl");
    let source = fs::read_to_string(path).unwrap();
    assert!(source.matches("var<storage").count() <= 8);
}
