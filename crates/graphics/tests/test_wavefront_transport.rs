use std::fs;
use std::path::PathBuf;

use inox_graphics::passes::pathtracing_common::{
    bounded_path_depth, ior_ratio, queue_capacity_for_resolution, queue_capacity_u32,
    reflect_direction, refract_direction, roulette_survival_probability, wavefront_bounce_stages,
    RayQueue, MAX_WAVEFRONT_QUEUE_CAPACITY,
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
    assert!(next.contains("let material_id = u32(material_value);"));
    assert!(!next.contains("bitcast<i32>(bitcast<u32>(gbuffer_material[offset]))"));
}

#[test]
fn wavefront_buffers_keep_full_resolution_with_non_native_render_scale() {
    let dimensions = (1920, 1080);
    let full_capacity = queue_capacity_for_resolution(dimensions, 1.0, usize::MAX);
    let scaled_capacity = queue_capacity_for_resolution(dimensions, 0.5, usize::MAX);
    assert!(scaled_capacity < full_capacity);
    assert_eq!(
        queue_capacity_for_resolution((1920, 1080), 1.0, MAX_WAVEFRONT_QUEUE_CAPACITY),
        2_073_600
    );
    assert_eq!(MAX_WAVEFRONT_QUEUE_CAPACITY, 2_097_152);
    assert_eq!(
        queue_capacity_for_resolution((3840, 2160), 1.0, MAX_WAVEFRONT_QUEUE_CAPACITY),
        MAX_WAVEFRONT_QUEUE_CAPACITY
    );
    assert_eq!(queue_capacity_u32(usize::MAX), u32::MAX);

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
        assert!(source.contains("queue_capacity_for_resolution"));
        assert!(!source.contains("usize::MAX"));
    }
    let transparency_pass = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/passes/compute_wavefront_transparency.rs"),
    )
    .unwrap();
    assert!(transparency_pass.contains("MAX_WAVEFRONT_QUEUE_CAPACITY"));
    assert!(transparency_pass.contains("saturating_mul(4)"));
    assert!(!transparency_pass.contains("MAX_TRANSPARENT_QUEUE_CAPACITY"));
    assert!(!transparency_pass.contains("1_048_576"));

    let denoise_pass = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/passes/compute_wavefront_denoise.rs"),
    )
    .unwrap();
    assert!(denoise_pass.contains("let (width, height) = self.dimensions"));
    assert!(!denoise_pass.contains("render_scale()"));
    let composite = shader("compute_wavefront_composite.wgsl");
    let temporal = shader("compute_wavefront_denoise_temporal.wgsl");
    let bilateral = shader("compute_wavefront_denoise_bilateral.wgsl");
    let upscale = shader("compute_wavefront_upscale.wgsl");
    for source in [&composite, &temporal, &bilateral, &upscale] {
        assert!(source.contains("u32(constant_data.screen_width)"));
        assert!(!source.contains("screen_width * constant_data.render_scale"));
    }
    assert!(upscale.contains("let source_index = id.y * width + id.x"));
}

#[test]
fn transparency_uses_full_resolution_pixel_indexing() {
    let source = shader("compute_wavefront_transparency.wgsl");
    assert!(source.contains("let width = u32(constant_data.screen_width);"));
    assert!(source.contains("let height = u32(constant_data.screen_height);"));
    assert!(!source.contains("screen_width * constant_data.render_scale"));
    assert!(!source.contains("screen_height * constant_data.render_scale"));
}

#[test]
fn entering_and_exiting_ior_ratios_are_reciprocal() {
    let ior = 1.5;
    assert!((ior_ratio(true, ior) - (1.0 / ior)).abs() < f32::EPSILON);
    assert!((ior_ratio(false, ior) - ior).abs() < f32::EPSILON);
    assert_eq!(ior_ratio(true, f32::NAN), 1.0 / 1.5);
    let source = shader("compute_wavefront_next_bounce.wgsl");
    assert!(source.contains("let eta_i = select(ior, 1.0, front_face);"));
    assert!(source.contains("let eta_t = select(1.0, ior, front_face);"));
    assert!(source.contains("let eta = eta_i / eta_t;"));
}

#[test]
fn invalid_material_ids_are_rejected_before_packed_access() {
    let surface = shader("compute_wavefront_surface.wgsl");
    let bounce = shader("compute_wavefront_next_bounce.wgsl");
    let transparency = shader("compute_wavefront_transparency.wgsl");
    assert!(surface.contains("material_index == INVALID_INDEX"));
    assert!(surface.contains("material_index >= i32(MAX_NUM_MATERIALS)"));
    assert!(surface.contains("write_empty(pixel);"));
    for source in [&bounce, &transparency] {
        assert!(source.contains("material_value != material_value"));
        assert!(source.contains("material_value >= f32(MAX_NUM_MATERIALS)"));
        assert!(source.contains("let material_id = u32(material_value);"));
    }
    assert!(
        bounce.find("material_value != material_value").unwrap()
            < bounce.find("load_material(material_id, 16u)").unwrap()
    );
    assert!(
        transparency
            .find("material_value != material_value")
            .unwrap()
            < transparency.find("table_offset(8u) + material_id").unwrap()
    );
}

#[test]
fn gpu_queue_overflow_has_an_explicit_readback_contract() {
    let common = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/passes/pathtracing_common.rs"),
    )
    .unwrap();
    let render_context = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../render/src/common/render_context.rs"),
    )
    .unwrap();
    let buffer_ref = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../render/src/common/buffer_ref.rs"),
    )
    .unwrap();
    let viewer = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../plugins/viewer/src/viewer.rs"),
    )
    .unwrap();
    assert!(common.contains("read_wavefront_queue_diagnostics"));
    assert!(common.contains("report_wavefront_queue_diagnostics"));
    assert!(common.contains("diagnostic.header.is_overflowed()"));
    assert!(common.contains("read_buffer_as::<QueueHeader>"));
    assert!(render_context.contains("read_buffer_as<T>"));
    assert!(render_context.contains("read_from_gpu_via_staging_as"));
    assert!(render_context.contains("add_post_submit_callback"));
    assert!(viewer.contains("add_post_submit_callback(report_wavefront_queue_diagnostics)"));
    assert!(buffer_ref.contains("MAP_READ | wgpu::BufferUsages::COPY_DST"));
    assert!(buffer_ref.contains("copy_buffer_to_buffer"));
    assert!(buffer_ref.contains("Option<Result<(), wgpu::BufferAsyncError>>"));
    assert!(buffer_ref.contains("if let Err(error) = render_core_context.device.poll"));
    assert!(buffer_ref.contains("Some(Err(error))"));
    assert!(buffer_ref.contains("GPU staging readback mapping did not complete"));
    assert!(!buffer_ref.contains("expect(\"Failed to map GPU staging buffer"));
    assert!(common.contains("pub fn is_overflowed(self) -> bool"));
}
