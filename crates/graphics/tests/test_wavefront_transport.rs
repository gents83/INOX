use std::fs;
use std::path::PathBuf;

use inox_graphics::passes::pathtracing_common::{
    bounded_path_depth, reflect_direction, refract_direction, roulette_survival_probability,
    RayQueue,
};

fn shader(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data_raw/shaders/wgsl")
        .join(name);
    fs::read_to_string(path).unwrap()
}

#[test]
fn empty_queue_terminates_dispatch_work() {
    let queue = RayQueue::<u32>::with_capacity(4);
    assert!(queue.is_empty());
    assert_eq!(queue.header().count, 0);
    assert!(shader("compute_wavefront_dispatch.wgsl").contains("workgroups"));
    assert!(shader("compute_wavefront_dispatch.wgsl").contains("select(0u, 1u, count > 0u)"));
}

#[test]
fn depth_and_roulette_are_bounded() {
    assert_eq!(bounded_path_depth(0), 0);
    assert_eq!(bounded_path_depth(99), 8);
    for throughput in [[0.0, 0.0, 0.0], [0.25, 0.5, 2.0], [f32::NAN, 1.0, 0.0]] {
        let probability = roulette_survival_probability(throughput);
        assert!((0.05..=0.99).contains(&probability));
    }
    let source = shader("compute_wavefront_next_bounce.wgsl");
    assert!(source.contains("constant_data.max_path_depth"));
    assert!(source.contains("constant_data.min_roulette_depth"));
    assert!(source.contains("throughput /= survival"));
}

#[test]
fn reflection_and_refraction_cover_tir_without_nan() {
    assert_eq!(
        reflect_direction([0.0, -1.0, 0.0], [0.0, 1.0, 0.0]),
        [0.0, 1.0, 0.0]
    );
    assert!(refract_direction([0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 1.0 / 1.5).is_some());
    assert!(refract_direction([0.99, 0.0, -0.14], [0.0, 0.0, 1.0], 1.5).is_none());
    let source = shader("compute_wavefront_next_bounce.wgsl");
    assert!(source.contains("candidate_refraction"));
    assert!(source.contains("let tir"));
    assert!(source.contains("safe_normalize"));
}

#[test]
fn environment_fallback_and_separate_accumulators_are_contracts() {
    let shade = shader("compute_wavefront_shade.wgsl");
    assert!(shade.contains("environment_texture_index"));
    assert!(shade.contains("sample_texture"));
    assert!(shade.contains("analytic_sky"));
    assert!(shade.contains("add_indirect"));
    let composite = shader("compute_wavefront_composite.wgsl");
    assert!(composite.contains("direct_accumulator"));
    assert!(composite.contains("indirect_accumulator"));
    assert!(composite.contains("sanitize_radiance"));
}

#[test]
fn queue_reset_happens_before_each_append_stage() {
    let next = shader("compute_wavefront_next_bounce.wgsl");
    let traversal = shader("compute_wavefront_traversal.wgsl");
    assert!(next.contains("atomicAdd(&next_queue_header[0].count"));
    assert!(traversal.contains("atomicAdd(&hit_queue_header[0].count"));
    assert!(traversal.contains("atomicStore(&hit_queue_header[0].overflowed"));
    let traversal_pass = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/passes/compute_wavefront_traversal.rs"),
    )
    .unwrap();
    assert!(traversal_pass.contains("header[0].count = 0"));
}
