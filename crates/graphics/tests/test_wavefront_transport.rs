use std::fs;
use std::path::PathBuf;

use inox_graphics::passes::pathtracing_common::{
    bounded_path_depth, ior_ratio, queue_capacity_for_resolution, queue_capacity_limits_for_extent,
    queue_capacity_u32, queue_diagnostic_marker_state, queue_fallback_capacity_for_adapter,
    reflect_direction, refract_direction, roulette_survival_probability, wavefront_bounce_queue_id,
    wavefront_bounce_stages, wavefront_queue_memory_bytes, QueueHeader, QueueOverflowDiagnostic,
    RayQueue, MAX_WAVEFRONT_QUEUE_CAPACITY, WAVEFRONT_QUEUE_FALLBACK_CAPACITY,
    WAVEFRONT_QUEUE_GPU_BYTES_PER_CAPACITY_ENTRY, WAVEFRONT_QUEUE_HOST_BYTES_PER_CAPACITY_ENTRY,
    WAVEFRONT_QUEUE_MEMORY_BUDGET_BYTES,
};
#[cfg(not(target_arch = "wasm32"))]
use inox_graphics::passes::pathtracing_common::{
    WAVEFRONT_NATIVE_QUEUE_FALLBACK_CAPACITY, WAVEFRONT_NATIVE_QUEUE_MEMORY_BUDGET_BYTES,
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
fn runtime_bounce_controls_keep_the_fixed_eight_stage_graph() {
    let stages = wavefront_bounce_stages();
    assert_eq!(stages.len(), 8);
    assert_eq!(bounded_path_depth(0), 0);
    assert_eq!(bounded_path_depth(1), 1);
    assert_eq!(bounded_path_depth(8), 8);
    assert_eq!(bounded_path_depth(9), 8);
    let constant_data = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../render/src/data/constant_data.rs"),
    )
    .unwrap();
    assert!(constant_data.contains("pub fn set_num_bounces"));
    assert!(constant_data.contains("let n = n.min(8)"));
    assert!(constant_data.contains("pub fn set_max_path_depth"));
    assert!(constant_data.contains("let depth = depth.min(8)"));
}

#[test]
fn generated_bounces_ping_pong_two_ray_queues_without_aliasing_a_stage() {
    let stages = wavefront_bounce_stages();
    let queues = stages
        .iter()
        .map(|stage| stage.bindings.destination_ray_queue)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(queues.len(), 2);
    assert_eq!(queues.len(), inox_graphics::WAVEFRONT_BOUNCE_QUEUE_COUNT);
    assert_eq!(wavefront_bounce_queue_id(1), wavefront_bounce_queue_id(3));
    assert_eq!(wavefront_bounce_queue_id(2), wavefront_bounce_queue_id(4));
    assert_ne!(wavefront_bounce_queue_id(1), wavefront_bounce_queue_id(2));
    for stage in &stages {
        assert_ne!(
            stage.bindings.source_ray_queue, stage.bindings.destination_ray_queue,
            "a bounce must not read and write the same ray queue"
        );
        assert_ne!(
            stage.bindings.destination_ray_header, stage.destination_dispatch_args,
            "stage header and dispatch args must remain distinct"
        );
    }
    for pair in stages.windows(2) {
        assert_eq!(
            pair[1].bindings.source_ray_queue,
            pair[0].bindings.destination_ray_queue
        );
    }
}

#[test]
fn default_ray_queue_allocation_has_a_bounded_memory_budget() {
    let stages = wavefront_bounce_stages();
    let mut queues = std::collections::BTreeSet::from([inox_graphics::WAVEFRONT_PRIMARY_QUEUE_ID]);
    queues.extend(
        stages
            .iter()
            .map(|stage| stage.bindings.destination_ray_queue),
    );
    let capacity = queue_capacity_for_resolution(
        1920,
        1080,
        std::mem::size_of::<inox_graphics::RayQueueRecord>() as u32,
        1,
        inox_graphics::portable_queue_capacity_limits(),
    )
    .unwrap() as usize;
    let bytes = queues.len() * capacity * std::mem::size_of::<inox_graphics::RayQueueRecord>();
    assert_eq!(queues.len(), 3, "primary plus two ping-pong queues");
    assert!(
        bytes <= WAVEFRONT_QUEUE_MEMORY_BUDGET_BYTES,
        "default GPU ray queue allocation was {bytes} bytes"
    );
    assert!(
        wavefront_queue_memory_bytes(capacity) <= WAVEFRONT_QUEUE_MEMORY_BUDGET_BYTES,
        "default host+GPU queue allocation exceeded the aggregate budget: {} > {}",
        wavefront_queue_memory_bytes(capacity),
        WAVEFRONT_QUEUE_MEMORY_BUDGET_BYTES
    );
}

#[test]
fn aggregate_queue_fallback_budget_includes_all_live_queue_classes() {
    let capacity = WAVEFRONT_QUEUE_FALLBACK_CAPACITY;
    assert!(capacity < MAX_WAVEFRONT_QUEUE_CAPACITY);
    assert!(wavefront_queue_memory_bytes(capacity) <= WAVEFRONT_QUEUE_MEMORY_BUDGET_BYTES);
    assert_eq!(
        WAVEFRONT_QUEUE_GPU_BYTES_PER_CAPACITY_ENTRY,
        3 * std::mem::size_of::<inox_graphics::RayQueueRecord>()
            + std::mem::size_of::<inox_graphics::HitRecord>()
            + 4 * std::mem::size_of::<inox_graphics::TransparentHitRecord>()
    );
    assert_eq!(
        WAVEFRONT_QUEUE_HOST_BYTES_PER_CAPACITY_ENTRY,
        WAVEFRONT_QUEUE_GPU_BYTES_PER_CAPACITY_ENTRY
    );
    assert_eq!(
        wavefront_queue_memory_bytes(capacity),
        capacity
            * (WAVEFRONT_QUEUE_GPU_BYTES_PER_CAPACITY_ENTRY
                + WAVEFRONT_QUEUE_HOST_BYTES_PER_CAPACITY_ENTRY)
    );

    let limits = inox_graphics::portable_queue_capacity_limits();
    let pixels = queue_capacity_for_resolution(1920, 1080, 64, 1, limits).unwrap();
    let hit = queue_capacity_for_resolution(1920, 1080, 32, 1, limits).unwrap();
    let transparent = queue_capacity_for_resolution(1920, 1080, 40, 4, limits).unwrap();
    assert_eq!(pixels as usize, capacity);
    assert_eq!(hit as usize, capacity);
    assert_eq!(transparent as usize, capacity);
    assert!(wavefront_queue_memory_bytes(capacity) <= 512 * 1024 * 1024);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_default_dimensions_fit_queue_budget_when_adapter_limits_permit() {
    assert_eq!(
        queue_fallback_capacity_for_adapter(true),
        WAVEFRONT_NATIVE_QUEUE_FALLBACK_CAPACITY
    );
    assert_eq!(
        queue_fallback_capacity_for_adapter(false),
        WAVEFRONT_QUEUE_FALLBACK_CAPACITY
    );
    let dimensions = (1900, 1037);
    let limits = inox_render::QueueCapacityLimits {
        max_storage_buffer_binding_size: WAVEFRONT_NATIVE_QUEUE_MEMORY_BUDGET_BYTES as u64,
        max_buffer_size: WAVEFRONT_NATIVE_QUEUE_MEMORY_BUDGET_BYTES as u64,
        fallback_capacity: WAVEFRONT_NATIVE_QUEUE_FALLBACK_CAPACITY as u32,
    };
    let primary = queue_capacity_for_resolution(
        dimensions.0,
        dimensions.1,
        std::mem::size_of::<inox_graphics::RayQueueRecord>() as u32,
        1,
        limits,
    )
    .unwrap() as usize;
    let hit = queue_capacity_for_resolution(
        dimensions.0,
        dimensions.1,
        std::mem::size_of::<inox_graphics::HitRecord>() as u32,
        1,
        limits,
    )
    .unwrap() as usize;
    let transparency = queue_capacity_for_resolution(
        dimensions.0,
        dimensions.1,
        std::mem::size_of::<inox_graphics::TransparentHitRecord>() as u32,
        4,
        limits,
    )
    .unwrap() as usize;
    let pixels = dimensions.0 as usize * dimensions.1 as usize;

    assert!(
        primary >= pixels,
        "native primary queue fits without truncation"
    );
    let maximize_dimensions = (1916, 1125);
    let maximize_primary = queue_capacity_for_resolution(
        maximize_dimensions.0,
        maximize_dimensions.1,
        std::mem::size_of::<inox_graphics::RayQueueRecord>() as u32,
        1,
        limits,
    )
    .unwrap() as usize;
    assert!(
        maximize_primary >= 2_155_500,
        "discrete-native maximize primary queue must not truncate: {maximize_primary}"
    );
    assert!(hit >= pixels, "native hit queue fits without truncation");
    assert!(
        transparency >= pixels,
        "native transparency queue fits without truncation"
    );
    assert!(
        wavefront_queue_memory_bytes(pixels) <= WAVEFRONT_NATIVE_QUEUE_MEMORY_BUDGET_BYTES,
        "native host+GPU queue estimate must fit without overflow"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn large_native_extent_uses_conservative_queue_fallback() {
    let limits = inox_render::QueueCapacityLimits {
        max_storage_buffer_binding_size: WAVEFRONT_NATIVE_QUEUE_MEMORY_BUDGET_BYTES as u64,
        max_buffer_size: WAVEFRONT_NATIVE_QUEUE_MEMORY_BUDGET_BYTES as u64,
        fallback_capacity: WAVEFRONT_NATIVE_QUEUE_FALLBACK_CAPACITY as u32,
    };
    let maximize = queue_capacity_limits_for_extent(limits, 1916, 1125);
    let large = queue_capacity_limits_for_extent(limits, 3820, 2117);
    let restored = queue_capacity_limits_for_extent(limits, 1920, 1080);
    assert_eq!(
        maximize.fallback_capacity as usize,
        WAVEFRONT_NATIVE_QUEUE_FALLBACK_CAPACITY
    );
    assert_eq!(
        large.fallback_capacity as usize,
        WAVEFRONT_QUEUE_FALLBACK_CAPACITY
    );
    assert_eq!(
        restored.fallback_capacity as usize,
        WAVEFRONT_NATIVE_QUEUE_FALLBACK_CAPACITY
    );
    let primary = queue_capacity_for_resolution(
        3820,
        2117,
        std::mem::size_of::<inox_graphics::RayQueueRecord>() as u32,
        1,
        large,
    )
    .unwrap() as usize;
    assert_eq!(primary, WAVEFRONT_QUEUE_FALLBACK_CAPACITY);
    assert!(primary < 3820usize * 2117);
}

#[test]
fn inactive_wavefront_passes_reset_headers_and_indirect_arguments() {
    let pass_sources = [
        "compute_wavefront_primary.rs",
        "compute_wavefront_dispatch.rs",
        "compute_wavefront_traversal.rs",
        "compute_wavefront_next_bounce.rs",
        "compute_wavefront_transparency.rs",
    ];
    for pass in pass_sources {
        let source = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src/passes")
                .join(pass),
        )
        .unwrap();
        assert!(
            source.contains("fn reset(&mut self, render_context: &RenderContext)"),
            "{pass}"
        );
        assert!(source.contains("update_buffer"), "{pass}");
    }
    let dispatch = shader("compute_wavefront_dispatch.wgsl");
    assert!(dispatch.contains("select(0u, 1u, count > 0u)"));
    assert!(dispatch.contains("min(atomicLoad(&queue_header[0].count), queue_header[0].capacity)"));
    let render_context = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../render/src/common/render_context.rs"),
    )
    .unwrap();
    assert!(render_context.contains("pass.reset(self)"));
}

#[test]
fn shared_hit_header_resets_immediately_before_each_traversal_dispatch() {
    let source = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/passes/compute_wavefront_traversal.rs"),
    )
    .unwrap();
    let update = source.find("fn update(").unwrap();
    let update_source = &source[update..];
    let reset = update_source
        .find("self.reset_hit_queue(render_context);")
        .expect("traversal update must reset the shared hit header");
    let dispatch = update_source
        .find("dispatch_indirect(")
        .expect("traversal update must dispatch after resetting the header");
    assert!(reset < dispatch);
    assert!(update_source[..reset].contains("self.dispatch_args.read().unwrap().is_empty()"));
    assert!(update_source[reset..dispatch].contains("let buffers = render_context.buffers();"));

    let stages = wavefront_bounce_stages();
    assert_eq!(stages.len(), 8);
    assert!(stages
        .iter()
        .all(|stage| stage.bindings.source_hit_header == inox_graphics::WAVEFRONT_HIT_HEADER_ID));
}

#[test]
fn packed_normals_use_numeric_f32_encoding() {
    let primary = shader("compute_wavefront_primary.wgsl");
    let direct = shader("compute_wavefront_direct_candidates.wgsl");
    let visibility = shader("compute_wavefront_visibility.wgsl");
    let next = shader("compute_wavefront_next_bounce.wgsl");
    for source in [&primary, &direct, &visibility, &next] {
        assert!(source.contains("unpack2x16float(u32(gbuffer_world"));
        assert!(!source.contains("bitcast<u32>(gbuffer_world"));
    }
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
fn bounce_dispatches_consume_shared_hit_args_and_chain_queues() {
    let stages = wavefront_bounce_stages();
    assert_eq!(stages.len(), 8);
    for stage in &stages {
        assert_eq!(
            stage.bindings.dispatch_args,
            inox_graphics::WAVEFRONT_HIT_DISPATCH_ARGS_ID
        );
    }
    for pair in stages.windows(2) {
        assert_eq!(
            pair[1].bindings.source_ray_queue,
            pair[0].bindings.destination_ray_queue
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
    assert!(source.contains("bsdf * cosine_out * eta_scale / (p_lobe * p_direction)"));
    assert!(source.contains("finite_scalar"));
    assert!(source.contains("finite_vector"));
    assert!(source.contains("if (tir) {"));
    assert!(source.contains("throughput /= survival"));
}

#[test]
fn environment_index_bounds_preserve_valid_black_textures() {
    let shade = shader("compute_wavefront_shade.wgsl");
    assert!(shade.contains("texture_index >= MAX_NUM_TEXTURES"));
    assert!(shade.contains("return sanitize_vector(sampled)"));
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
fn queue_header_resets_upload_before_dispatch() {
    for pass in [
        "compute_wavefront_primary.rs",
        "compute_wavefront_traversal.rs",
        "compute_wavefront_next_bounce.rs",
        "compute_wavefront_transparency.rs",
    ] {
        let source = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src/passes")
                .join(pass),
        )
        .unwrap();
        assert!(
            source.contains("render_context.update_buffer(&mut *header);")
                || source.contains("render_context.update_buffer(&mut *header);"),
            "{pass} must upload its CPU queue-header reset"
        );
        assert!(
            !source.contains("render_context.mark_as_dirty(header.buffer_id())"),
            "{pass} must not defer its queue-header reset to binding initialization"
        );
    }

    let binding_buffers = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../render/src/common/binding_buffers.rs"),
    )
    .unwrap();
    assert!(binding_buffers.contains("pub fn update_buffer<T>"));
    assert!(binding_buffers.contains("wgpu::BufferUsages::COPY_SRC"));
    let buffer_ref = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../render/src/common/buffer_ref.rs"),
    )
    .unwrap();
    assert!(buffer_ref.contains("self.offset = 0;"));
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
fn queue_pass_error_fallbacks_use_negotiated_nonzero_capacity() {
    for pass in [
        "compute_wavefront_primary.rs",
        "compute_wavefront_traversal.rs",
        "compute_wavefront_next_bounce.rs",
        "compute_wavefront_transparency.rs",
    ] {
        let source = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src/passes")
                .join(pass),
        )
        .unwrap();
        assert!(
            source.contains(".unwrap_or(capacity_limits.fallback_capacity.max(1)) as usize;"),
            "{pass} must recover with its negotiated nonzero fallback capacity"
        );
        assert!(
            !source.contains("MAX_WAVEFRONT_QUEUE_CAPACITY.min(WAVEFRONT_QUEUE_FALLBACK_CAPACITY)"),
            "{pass} must not recover with the portable fallback policy"
        );
    }
}

#[test]
fn wavefront_buffers_keep_full_resolution_with_non_native_render_scale() {
    let dimensions = (1920, 1080);
    let limits = inox_render::QueueCapacityLimits {
        fallback_capacity: u32::MAX,
        ..Default::default()
    };
    let full_capacity =
        queue_capacity_for_resolution(dimensions.0, dimensions.1, 16, 1, limits).unwrap();
    let scaled_capacity = queue_capacity_for_resolution(960, 540, 16, 1, limits).unwrap();
    assert!(scaled_capacity < full_capacity);
    assert_eq!(
        queue_capacity_for_resolution(
            1920,
            1080,
            16,
            1,
            inox_render::QueueCapacityLimits::default()
        )
        .unwrap(),
        2_073_600
    );
    assert_eq!(MAX_WAVEFRONT_QUEUE_CAPACITY, 2_796_202);
    assert_eq!(
        queue_capacity_for_resolution(
            3840,
            2160,
            16,
            1,
            inox_render::QueueCapacityLimits::default()
        )
        .unwrap(),
        2_097_152
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
        assert!(source.contains("capacity_limits.fallback_capacity.max(1)"));
        assert!(!source.contains("usize::MAX"));
    }
    let transparency_pass = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/passes/compute_wavefront_transparency.rs"),
    )
    .unwrap();
    assert!(transparency_pass.contains("capacity_limits.fallback_capacity.max(1)"));
    assert!(transparency_pass.contains("checked_mul(4)"));
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
fn direct_lighting_and_ao_use_full_resolution_pixel_strides() {
    for name in [
        "compute_wavefront_direct_candidates.wgsl",
        "compute_wavefront_visibility.wgsl",
        "compute_wavefront_ao_resolve.wgsl",
    ] {
        let source = shader(name);
        assert!(source.contains("u32(constant_data.screen_width)"), "{name}");
        assert!(
            source.contains("u32(constant_data.screen_height)"),
            "{name}"
        );
        assert!(
            !source.contains("screen_width * constant_data.render_scale"),
            "{name}"
        );
        assert!(
            !source.contains("screen_height * constant_data.render_scale"),
            "{name}"
        );
        assert!(
            source.contains("let pixel = id.y * width + id.x")
                || source.contains("let pixel = id.y * width + u32(id.x)"),
            "{name}"
        );
    }
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
    let shade = shader("compute_wavefront_shade.wgsl");
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
    assert!(shade.contains("material_index == INVALID_INDEX"));
    assert!(shade.contains("material_index < 0"));
    assert!(shade.contains("material_index >= i32(MAX_NUM_MATERIALS)"));
    assert!(!shade.contains("max(bitcast<i32>(scene_data[mesh_base + 3u]), 0)"));
    assert!(
        shade.find("material_index == INVALID_INDEX").unwrap()
            < shade.find("load_material(material_id").unwrap()
    );
}

#[test]
fn packed_scene_wavefront_stages_use_bounded_table_access_helpers() {
    for name in [
        "compute_wavefront_surface.wgsl",
        "compute_wavefront_traversal.wgsl",
        "compute_wavefront_visibility.wgsl",
        "compute_wavefront_direct_candidates.wgsl",
        "compute_wavefront_transparency.wgsl",
        "compute_wavefront_shade.wgsl",
        "compute_wavefront_next_bounce.wgsl",
    ] {
        let source = shader(name);
        assert!(source.contains("#import \"packed_scene.inc\""), "{name}");
        assert!(source.contains("packed_scene_valid()"), "{name}");
    }
    let packed = shader("packed_scene.inc");
    assert!(packed.contains("scene_data[0u] != PACKED_BUFFER_MAGIC"));
    assert!(packed.contains("scene_data[1u] != PACKED_BUFFER_VERSION"));
    assert!(packed.contains("words != storage"));
    assert!(packed.contains("packed_table_expected_stride"));
    assert!(packed.contains("left_end = left_offset + left_count"));
    assert!(packed.contains("fn packed_table_range_valid"));
    assert!(packed.contains("fn range_has_words_at"));
    assert!(packed.contains("fn range_has_indexed_words"));
    assert!(packed.contains("fn scene_relative_index_valid"));
    let surface = shader("compute_wavefront_surface.wgsl");
    assert!(surface.contains("let encoded_instance = visibility >> 8u;"));
    assert!(surface.contains("if (encoded_instance == 0u) { write_empty(pixel); return; }"));
    assert!(!surface.contains("encoded_instance >= 0x00FFu"));
    assert!(
        surface.find("encoded_instance == 0u").unwrap()
            < surface
                .find("let instance_id = encoded_instance - 1u")
                .unwrap()
    );
    assert!(surface.contains("!scene_index_valid(3u, instance_id, 4u)"));
    assert!(surface.contains("!scene_index_valid(4u, transform_id, 16u)"));
    assert!(surface.contains("!scene_index_valid(5u, mesh_index, 16u)"));
    assert!(surface.contains("!scene_index_valid(6u, meshlet_index, 20u)"));
    assert!(surface
        .contains("!range_has_indexed_words(0u, meshlet_indices_offset, primitive_id, 3u, 3u)"));
    assert!(surface.contains("!range_has_words_at(1u, vertex_offset, i0, 1u)"));
    let traversal = shader("compute_wavefront_traversal.wgsl");
    assert!(traversal.contains(
        "scene_relative_index_valid(6u, mesh.meshlets_offset, u32(node.reference), 20u)"
    ));
    assert!(traversal.contains("range_has_indexed_words(0u, indices_offset, triangle, 3u, 3u)"));
    assert!(traversal.contains("range_has_words_at(1u, mesh.vertices_position_offset, i0, 1u)"));
    let visibility = shader("compute_wavefront_visibility.wgsl");
    assert!(
        visibility.contains("scene_relative_index_valid(6u, meshlet_start, relative_meshlet, 20u)")
    );
    assert!(visibility.contains("range_has_indexed_words(0u, io, tri, 3u, 3u)"));
    assert!(visibility.contains("range_has_words_at(1u, vbase, i0, 1u)"));
}

#[test]
fn packed_scene_material_and_visibility_reads_are_rejected_before_access() {
    let surface = shader("compute_wavefront_surface.wgsl");
    let transparency = shader("compute_wavefront_transparency.wgsl");
    let shade = shader("compute_wavefront_shade.wgsl");
    let next = shader("compute_wavefront_next_bounce.wgsl");
    let visibility = shader("compute_wavefront_visibility.wgsl");
    let direct = shader("compute_wavefront_direct_candidates.wgsl");
    let traversal = shader("compute_wavefront_traversal.wgsl");
    assert!(direct.contains("!packed_scene_valid()"));
    assert!(direct.contains(
        "!range_has_words(11u, record_relative, PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS)"
    ));
    assert!(direct.contains("!scene_index_valid(10u, light_id, 16u)"));
    assert!(traversal.contains("if (!packed_scene_valid()) { return; }"));
    assert!(transparency.contains("if (!scene_index_valid(8u, material_id, 48u)) { return; }"));
    assert!(shade.contains("!scene_index_valid(8u, u32(material_index), 48u)"));
    assert!(next.contains("if (!scene_index_valid(8u, material_id, 48u)) { return; }"));
    assert!(visibility.contains("if (!scene_index_valid(3u, instance, 4u)) { continue; }"));
    assert!(visibility.contains("if (!scene_index_valid(4u, transform_id, 16u) || !scene_index_valid(5u, mesh_index, 16u)) { continue; }"));
    assert!(visibility.contains("!range_has_words(0u, io, index_count)"));
    assert!(visibility.contains("!range_has_words_at(1u, vbase, i0, 1u)"));
    assert!(
        visibility.contains("scene_relative_index_valid(6u, meshlet_start, relative_meshlet, 20u)")
    );
    for source in [
        &surface,
        &traversal,
        &visibility,
        &direct,
        &transparency,
        &shade,
        &next,
    ] {
        assert!(!surface_contains_unchecked_table_offset(source));
    }
}

fn surface_contains_unchecked_table_offset(source: &str) -> bool {
    source.contains("fn table_offset(kind: u32) -> u32 { return scene_data[4u + kind * 4u]; }")
}

#[test]
fn malformed_visibility_cannot_wrap_instance_id() {
    let source = shader("compute_wavefront_surface.wgsl");
    let guard = source
        .find("let encoded_instance = visibility >> 8u;")
        .expect("visibility decoder must exist");
    let subtraction = source
        .find("let instance_id = encoded_instance - 1u;")
        .expect("instance subtraction must exist");
    let guard_text = &source[guard..subtraction];
    assert!(guard_text.contains("encoded_instance == 0u"));
    assert!(!guard_text.contains("encoded_instance >= 0x00FFu"));
    assert!(!guard_text.contains("visibility >> 8u) - 1u"));
    assert!(source.contains("scene_index_valid(3u, instance_id, 4u)"));
    let traversal = shader("compute_wavefront_traversal.wgsl");
    assert!(traversal.contains("instance_id < 0x00FFFFFEu"));
    assert!(traversal.contains("((instance_id + 1u) << 8u)"));
}

#[test]
fn finalize_uses_active_screen_dimensions_for_accumulators() {
    let source = shader("finalize.wgsl");
    assert!(source.contains("let data_dimensions = dimensions;"));
    assert!(!source.contains("vec2<u32>(DEFAULT_WIDTH, DEFAULT_HEIGHT)"));
}

#[test]
fn transport_properties_cover_all_lobes_and_finite_termination_contracts() {
    let source = shader("compute_wavefront_next_bounce.wgsl");
    for incident in [[0.0, -1.0, 0.0], [0.6, -0.8, 0.0], [-0.8, -0.2, 0.3]] {
        let normal = [0.0, 1.0, 0.0];
        let reflected = reflect_direction(incident, normal);
        assert!(reflected.iter().all(|v| v.is_finite()));
        if let Some(refracted) = refract_direction(incident, normal, 1.0 / 1.5) {
            assert!(refracted.iter().all(|v| v.is_finite()));
        }
    }
    assert!(source.contains("cosine_sample(normal"));
    assert!(source.contains("cosine_sample(reflection"));
    assert!(source.contains("eta_scale = eta * eta"));
    assert!(source.contains("if (tir) {"));
    assert!(source.contains("p_direction <= MATH_EPSILON"));
    assert!(source.contains("throughput /= survival"));
    assert!(source.contains("finite_vector(throughput)"));
}

#[test]
fn direct_reservoir_normalizes_selected_sample_and_invalidates_history() {
    let source = shader("compute_wavefront_direct_candidates.wgsl");
    assert!(source.contains("previous.flags == constant_data.history_epoch"));
    assert!(source.contains("neighbor.flags == constant_data.history_epoch"));
    assert!(source.contains("selected * reservoir.weight_sum"));
    assert!(source.contains("f32(max(reservoir.candidate_count, 1u)) * selected_weight"));
    assert!(source.contains("MAX_RESERVOIR_CANDIDATES"));
    assert!(source.contains("add_candidate_count"));
    assert!(source.contains("valid_weight(reservoir.weight_sum)"));
    assert!(source.contains("finite_vector(candidate_estimate)"));
    assert!(source.contains("reservoir.flags = constant_data.history_epoch"));
    assert!(source.contains("finite_scalar(distance_sq)"));
}

#[test]
fn shade_clears_indirect_accumulator_and_rejects_nonfinite_inputs() {
    let shader_source = shader("compute_wavefront_shade.wgsl");
    assert!(shader_source.contains("arrayLength(&indirect_accumulator)"));
    assert!(shader_source.contains("finite_vector(ray.throughput.xyz)"));
    assert!(shader_source.contains("sanitize_vector(sampled)"));
    assert!(shader_source.contains("let emissive = sanitize_vector"));
    assert!(shader_source.contains("let base_color = sanitize_vector"));
    assert!(shader_source.contains("explicitly two-sided"));
    assert!(!shader_source.contains("vec3<f32>(0.0, 1.0, 0.0)), 0.0"));
    let pass = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/passes/compute_wavefront_shade.rs"),
    )
    .unwrap();
    assert!(pass.contains("fn reset(&mut self, render_context: &RenderContext)"));
    assert!(pass.contains("accumulator.fill(AccumulationRecord::default())"));
    assert!(pass.contains("render_context.update_buffer(&mut *accumulator)"));
}

#[test]
fn temporal_epoch_and_full_resolution_denoiser_contracts_are_explicit() {
    let common = shader("common.inc");
    let composite = shader("compute_wavefront_composite.wgsl");
    let direct = shader("compute_wavefront_direct_candidates.wgsl");
    assert!(common.contains("history_epoch: u32"));
    assert!(composite.contains("final_accumulator[pixel].flags == constant_data.history_epoch"));
    assert!(composite.contains("select(1u"));
    assert!(composite.contains("abs(value) < vec3<f32>(65504.0)"));
    assert!(direct.contains("reservoir.flags = constant_data.history_epoch"));
    for name in [
        "compute_wavefront_denoise_temporal.wgsl",
        "compute_wavefront_denoise_bilateral.wgsl",
    ] {
        let source = shader(name);
        assert!(source.contains("u32(constant_data.screen_width)"), "{name}");
        assert!(
            source.contains("u32(constant_data.screen_height)"),
            "{name}"
        );
        assert!(!source.contains("render_scale"), "{name}");
    }
    let temporal = shader("compute_wavefront_denoise_temporal.wgsl");
    assert!(temporal.contains("history_accumulator"));
    assert!(temporal.contains("previous.flags == constant_data.history_epoch"));
    assert!(temporal.contains("first_and_second_moment"));
    assert!(temporal.contains("previous_variance"));
}

#[test]
fn camera_resize_and_material_changes_advance_temporal_history_epoch() {
    let constant_data = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../render/src/data/constant_data.rs"),
    )
    .unwrap();
    let global_buffers = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../render/src/common/global_buffers.rs"),
    )
    .unwrap();
    assert!(constant_data.contains("history_epoch: u32"));
    assert!(constant_data.contains("pub fn invalidate_history"));
    assert!(constant_data.contains("history_changed"));
    assert!(constant_data.contains("self.invalidate_history(render_context)"));
    assert!(
        global_buffers
            .matches("invalidate_history(render_context)")
            .count()
            >= 6
    );
    assert!(global_buffers.contains("*light.data()"));
    assert!(global_buffers.contains("lights.push(light_id, *light_data)"));
    assert!(global_buffers.contains("if !removed"));
    assert!(global_buffers.contains("for_each_data"));
    assert!(global_buffers.contains("saturating_sub(1)"));
    let update_system = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../render/src/systems/update_system.rs"),
    )
    .unwrap();
    assert!(update_system.contains("if light.get().is_active()"));
    assert!(update_system.contains("if light.is_active()"));
    assert!(
        update_system
            .matches("remove_light(&self.render_context, id)")
            .count()
            >= 2
    );
    assert!(update_system.contains("set_light_index(index)"));
    assert!(update_system.contains("light.get_mut().invalidate()"));
    assert!(update_system.contains("light.invalidate()"));
    assert!(update_system.contains("DataTypeResourceEvent<Light>"));
}

#[test]
fn light_reactivation_rebinds_reused_gpu_index_and_invalidates_removal() {
    let global_buffers = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../render/src/common/global_buffers.rs"),
    )
    .unwrap();
    let light = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../render/src/resources/light.rs"),
    )
    .unwrap();
    assert!(global_buffers.contains("get_first_with_index_mut(light_id)"));
    assert!(global_buffers.contains("(inserted.start as u32, true)"));
    assert!(global_buffers.contains("index\n    }"));
    assert!(global_buffers.contains("light.set_light_index(index as _)"));
    assert!(global_buffers.contains("light.invalidate()"));
    assert!(global_buffers.contains("pub fn remove_light") && global_buffers.contains(") -> bool"));
    assert!(light.contains("self.light_index = INVALID_INDEX"));
    assert!(light.contains("fn is_initialized(&self) -> bool"));
    let update_system = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../render/src/systems/update_system.rs"),
    )
    .unwrap();
    // A's removal invalidates its resource before B can reuse the freed slot;
    // reactivation's update/add path writes the returned slot back to A.
    assert!(update_system.contains("set_light_index(index)"));
}

#[test]
fn queue_scheduling_does_not_cpu_gate_on_gpu_written_counts() {
    for pass in [
        "compute_wavefront_next_bounce.rs",
        "compute_wavefront_shade.rs",
    ] {
        let source = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src/passes")
                .join(pass),
        )
        .unwrap();
        assert!(!source.contains("header.count > 0"), "{pass}");
        assert!(!source.contains("header.count == 0"), "{pass}");
    }
}

#[test]
fn queue_capacity_is_checked_and_adapter_limited() {
    let limits = inox_render::QueueCapacityLimits {
        max_storage_buffer_binding_size: 64,
        max_buffer_size: 96,
        fallback_capacity: 32,
    };
    assert_eq!(
        queue_capacity_for_resolution(0, 1080, 16, 1, limits).unwrap(),
        0
    );
    assert_eq!(
        queue_capacity_for_resolution(1, 1, 16, 1, limits).unwrap(),
        1
    );
    assert_eq!(
        queue_capacity_for_resolution(100, 100, 16, 1, limits).unwrap(),
        4
    );
    assert!(matches!(
        queue_capacity_for_resolution(u32::MAX, u32::MAX, 16, 1, limits),
        Ok(4)
    ));
    assert!(matches!(
        queue_capacity_for_resolution(1, 1, 0, 1, limits),
        Err(inox_graphics::passes::pathtracing_common::CapacityError::ZeroRecordSize)
    ));
}

#[test]
fn cpu_queue_reports_dropped_records_and_resets_diagnostics() {
    let mut queue = RayQueue::with_capacity(1);
    assert!(queue.append(1u32));
    assert!(!queue.append(2u32));
    assert_eq!(queue.header().dropped_records, 1);
    queue.reset();
    assert_eq!(queue.header().dropped_records, 0);
    assert!(!queue.is_overflowed());
}

fn marker_diagnostics(
    count: usize,
    overflowed: bool,
    dropped_records: u32,
) -> Vec<QueueOverflowDiagnostic> {
    (0..count)
        .map(|index| QueueOverflowDiagnostic {
            name: format!("queue{index}"),
            requested_capacity: 8,
            actual_capacity: 8,
            overflowed,
            dropped_records,
            active_bounce_count: 2,
            header: QueueHeader {
                capacity: 8,
                overflowed: u32::from(overflowed),
                dropped_records,
                ..QueueHeader::default()
            },
        })
        .collect()
}

#[test]
fn queue_diagnostic_markers_require_all_clean_headers() {
    let state = queue_diagnostic_marker_state(&marker_diagnostics(11, false, 0), 11);
    assert!(state.complete);
    assert!(state.no_overflow);
}

#[test]
fn queue_diagnostic_markers_keep_complete_marker_for_overflow() {
    let overflowed = queue_diagnostic_marker_state(&marker_diagnostics(11, true, 0), 11);
    assert!(overflowed.complete);
    assert!(!overflowed.no_overflow);

    let dropped = queue_diagnostic_marker_state(&marker_diagnostics(11, false, 1), 11);
    assert!(dropped.complete);
    assert!(!dropped.no_overflow);
}

#[test]
fn queue_diagnostic_markers_reject_partial_readback() {
    let state = queue_diagnostic_marker_state(&marker_diagnostics(10, false, 0), 11);
    assert!(!state.complete);
    assert!(!state.no_overflow);
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
    assert!(common.contains("cfg(not(target_arch = \"wasm32\"))"));
    assert!(common.contains("Queue readback is unavailable on pinned WebGPU/WASM"));
    assert!(common.contains("Persistent overflow is deduplicated"));
    assert!(common.contains("read_buffer_as::<QueueHeader>"));
    assert!(common.contains("EXPECTED_WAVEFRONT_QUEUE_HEADER_COUNT"));
    assert!(common.contains("INOX_NATIVE_WAVEFRONT_QUEUE_HEADERS_FAILED"));
    assert!(common.contains("INOX_NATIVE_WAVEFRONT_QUEUE_HEADERS_COMPLETE"));
    assert!(common.contains("INOX_NATIVE_WAVEFRONT_QUEUE_NO_OVERFLOW"));
    assert!(!common.contains("INOX_NATIVE_WAVEFRONT_QUEUE_HEADERS_OK"));
    assert!(common.contains("diagnostics.iter().all"));
    assert!(common.contains("diagnostic.header.dropped_records == 0"));
    assert!(common.contains("diagnostics.len() != expected"));
    assert!(render_context.contains("read_buffer_as<T>"));
    assert!(render_context.contains("read_from_gpu_via_staging_as"));
    assert!(render_context.contains("add_post_submit_callback"));
    assert!(render_context.contains("let callbacks = self"));
    assert!(render_context.contains(".iter()"));
    assert!(render_context.contains(".cloned()"));
    assert!(viewer.contains("add_post_submit_callback(report_wavefront_queue_diagnostics)"));
    assert!(!viewer.contains("signal_browser_smoke_marker(\"queue_headers\""));
    assert!(!viewer.contains("signal_browser_smoke_marker(\"output\""));
    assert!(!viewer.contains("registration skipped on WebGPU/WASM"));
    assert!(viewer.contains("cfg(not(target_arch = \"wasm32\"))"));
    assert!(buffer_ref.contains("MAP_READ | wgpu::BufferUsages::COPY_DST"));
    assert!(buffer_ref.contains("copy_buffer_to_buffer"));
    assert!(buffer_ref.contains("Option<Result<(), wgpu::BufferAsyncError>>"));
    assert!(buffer_ref.contains("if let Err(error) = render_core_context.device.poll"));
    assert!(buffer_ref.contains("Some(Err(error))"));
    assert!(buffer_ref.contains("GPU staging readback mapping did not complete"));
    assert!(!buffer_ref.contains("expect(\"Failed to map GPU staging buffer"));
    assert!(common.contains("pub fn is_overflowed(self) -> bool"));
}
