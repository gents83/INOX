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
fn direct_candidate_shader_keeps_the_portable_storage_budget() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data_raw/shaders/wgsl/compute_wavefront_direct_candidates.wgsl");
    let source = fs::read_to_string(path).unwrap();
    assert!(source.matches("var<storage").count() <= 8);
}
