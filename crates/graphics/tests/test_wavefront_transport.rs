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

fn braced_block_after<'a>(source: &'a str, marker: &str) -> &'a str {
    let marker_start = source.find(marker).expect("shader marker must exist");
    let open = source[marker_start..]
        .find('{')
        .map(|offset| marker_start + offset)
        .expect("shader marker must open a block");
    let mut depth = 0usize;
    for (offset, character) in source[open..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[open..=open + offset];
                }
            }
            _ => {}
        }
    }
    panic!("shader block must close");
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
fn tir_forces_reflection_and_preserves_throughput() {
    let source = shader("compute_wavefront_next_bounce.wgsl");
    let tir_start = source.find("if (tir) {").unwrap();
    let lobe_start = source.find("} else if (selector < p_transmission").unwrap();
    let estimator_start = source.find("if (!tir) {").unwrap();
    let tir_block = braced_block_after(&source[tir_start..], "if (tir) {");
    let estimator_block = braced_block_after(&source[estimator_start..], "if (!tir) {");

    assert!(tir_start < lobe_start);
    assert!(lobe_start < estimator_start);
    assert!(tir_block.contains("direction = reflection;"));
    assert!(tir_block.contains("p_lobe = 1.0;"));
    assert!(tir_block.contains("p_direction = 1.0;"));
    assert!(tir_block.contains("throughput = ray.throughput.xyz;"));
    assert!(estimator_block.contains("throughput = ray.throughput.xyz * bsdf * cosine_out"));
}

#[test]
fn next_bounce_uses_explicit_lobe_pdf_and_finite_guards() {
    let source = shader("compute_wavefront_next_bounce.wgsl");
    assert!(source.contains("p_lobe"));
    assert!(source.contains("p_direction"));
    assert!(source.contains("bsdf * cosine_out / (p_lobe * p_direction)"));
    assert!(source.contains("finite_scalar"));
    assert!(source.contains("finite_vector"));
    assert!(source.contains("if (tir) {"));
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

#[test]
fn cpu_activation_uses_static_resources_not_gpu_queue_counts() {
    for pass in [
        "compute_wavefront_dispatch.rs",
        "compute_wavefront_next_bounce.rs",
        "compute_wavefront_shade.rs",
        "compute_wavefront_traversal.rs",
    ] {
        let source = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src/passes")
                .join(pass),
        )
        .unwrap();
        assert!(
            !source.contains("header.count > 0") && !source.contains("header.count == 0"),
            "{pass} must not use a GPU-written count as a CPU activation gate"
        );
    }
}

#[test]
fn material_ids_are_numeric_between_surface_and_bounce() {
    let surface = shader("compute_wavefront_surface.wgsl");
    let next = shader("compute_wavefront_next_bounce.wgsl");
    assert!(surface.contains("data_buffer_1[o] = f32(material_id)"));
    assert!(next.contains("u32(max(gbuffer_material[offset], 0.0))"));
    assert!(!next.contains("bitcast<i32>(bitcast<u32>(gbuffer_material[offset]))"));
}

#[test]
fn wavefront_buffers_keep_full_resolution_with_non_native_render_scale() {
    use inox_graphics::passes::pathtracing_common::queue_capacity_for_resolution;

    let dimensions = (1920, 1080);
    let full_capacity = queue_capacity_for_resolution(dimensions, 1.0, usize::MAX);
    let scaled_capacity = queue_capacity_for_resolution(dimensions, 0.5, usize::MAX);
    assert!(scaled_capacity < full_capacity);

    // Visibility, G-buffer, direct/indirect accumulators, and all wavefront
    // queues are full-resolution resources.  Render scale is reconstructed by
    // the final upscale pass, so queue capacity must not silently use it.
    for pass in [
        "compute_wavefront_primary.rs",
        "compute_wavefront_traversal.rs",
        "compute_wavefront_next_bounce.rs",
    ] {
        let source = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src/passes")
                .join(pass),
        )
        .unwrap();
        assert!(source.contains("queue_capacity_for_resolution(dimensions, 1.0"));
    }
    let upscale = shader("compute_wavefront_upscale.wgsl");
    assert!(upscale.contains("constant_data.render_scale"));
    assert!(upscale.contains("source_width"));
}
