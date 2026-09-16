use std::fs;
use std::path::PathBuf;

use inox_graphics::passes::pathtracing_common::{
    bounded_path_depth, reflect_direction, refract_direction, roulette_survival_probability,
    wavefront_bounce_stages, RayQueue,
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
fn bounce_dispatches_chain_through_previous_destination() {
    let stages = wavefront_bounce_stages();
    assert_eq!(stages.len(), 8);
    for pair in stages.windows(2) {
        assert_eq!(
            pair[1].bindings.dispatch_args,
            pair[0].destination_dispatch_args
        );
        assert_ne!(
            pair[1].bindings.dispatch_args,
            inox_graphics::WAVEFRONT_HIT_DISPATCH_ARGS_ID
        );
    }
}

#[test]
fn next_bounce_uses_explicit_lobe_pdf_and_finite_guards() {
    let source = shader("compute_wavefront_next_bounce.wgsl");
    assert!(source.contains("p_lobe"));
    assert!(source.contains("p_direction"));
    assert!(source.contains("bsdf * cosine_out / (p_lobe * p_direction)"));
    assert!(source.contains("finite_scalar"));
    assert!(source.contains("finite_vector"));
    assert!(source.contains("if (!tir"));
    assert!(source.contains("throughput /= survival"));
}

#[test]
fn environment_index_bounds_preserve_valid_black_textures() {
    let shade = shader("compute_wavefront_shade.wgsl");
    assert!(shade.contains("texture_index >= MAX_NUM_TEXTURES"));
    assert!(shade.contains("return max(sampled, vec3<f32>(0.0))"));
    assert!(!shade.contains("length(sampled) > MATH_EPSILON"));
}

#[test]
fn max_depth_clamps_roulette_depth_and_marks_constants_dirty() {
    let source = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../render/src/data/constant_data.rs"),
    )
    .unwrap();
    assert!(source
        .contains("let roulette_depth = self.min_roulette_depth.min(depth.saturating_sub(1));"));
    assert!(source.contains("self.min_roulette_depth = roulette_depth;"));
    assert!(source.contains("self.mark_as_dirty(render_context);"));
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
