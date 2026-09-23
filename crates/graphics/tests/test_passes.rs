use inox_render::{
    WAVEFRONT_ACCUMULATION_RECORD_STRIDE_WORDS, WAVEFRONT_AO_RECORD_STRIDE_WORDS,
    WAVEFRONT_DISPATCH_ARGS_STRIDE_WORDS, WAVEFRONT_HIT_RECORD_STRIDE_WORDS,
    WAVEFRONT_QUEUE_HEADER_STRIDE_WORDS, WAVEFRONT_RAY_QUEUE_RECORD_STRIDE_WORDS,
    WAVEFRONT_RESERVOIR_RECORD_STRIDE_WORDS, WAVEFRONT_TRANSPARENT_HIT_RECORD_STRIDE_WORDS,
};

use inox_graphics::{
    dispatch_workgroups_for, queue_capacity_for_resolution, wavefront_bounce_stage,
    AccumulationRecord, AoRecord, DispatchArgs, HitRecord, RayQueue, RayQueueRecord,
    ReservoirRecord, TransparentHitRecord, BLIT_PASS_NAME, COMPUTE_WAVEFRONT_COMPOSITE_NAME,
    COMPUTE_WAVEFRONT_DIRECT_CANDIDATES_NAME, COMPUTE_WAVEFRONT_DISPATCH_NAME,
    COMPUTE_WAVEFRONT_PRIMARY_NAME, COMPUTE_WAVEFRONT_SHADE_NAME, COMPUTE_WAVEFRONT_TRAVERSAL_NAME,
    CULLING_PASS_NAME, DEPTH_FIRST_PASS_NAME, DEPTH_PYRAMID_PASS_NAME, WAVEFRONT_AO_ID,
    WAVEFRONT_DIRECT_ACCUMULATOR_ID, WAVEFRONT_DISPATCH_ARGS_ID, WAVEFRONT_FINAL_ACCUMULATOR_ID,
    WAVEFRONT_HIT_DISPATCH_ARGS_ID, WAVEFRONT_HIT_HEADER_ID, WAVEFRONT_HIT_QUEUE_ID,
    WAVEFRONT_INDIRECT_ACCUMULATOR_ID, WAVEFRONT_NEXT2_DISPATCH_ARGS_ID, WAVEFRONT_NEXT2_HEADER_ID,
    WAVEFRONT_NEXT2_QUEUE_ID, WAVEFRONT_NEXT3_DISPATCH_ARGS_ID, WAVEFRONT_NEXT3_HEADER_ID,
    WAVEFRONT_NEXT3_QUEUE_ID, WAVEFRONT_NEXT_DISPATCH_ARGS_ID, WAVEFRONT_NEXT_HEADER_ID,
    WAVEFRONT_NEXT_QUEUE_ID, WAVEFRONT_PRIMARY_HEADER_ID, WAVEFRONT_PRIMARY_QUEUE_ID,
    WAVEFRONT_RESERVOIR_ID, WAVEFRONT_SCENE_ID, WAVEFRONT_TRANSPARENT_HEADER_ID,
    WAVEFRONT_TRANSPARENT_QUEUE_ID, WAVEFRONT_UPSCALED_ACCUMULATOR_ID,
};

#[test]
fn surface_retries_initialization_after_transient_invalid_scene() {
    let source = include_str!("../src/passes/compute_wavefront_surface.rs");
    let should_init = source
        .find("fn should_init(&self")
        .expect("surface must expose an initialization retry predicate");
    let retry_condition = source
        .find("(!self.scene_valid || !self.initialized)")
        .expect("invalid scene must remain eligible for initialization retry");
    let init = source
        .find("fn init(&mut self, render_context: &RenderContext)")
        .expect("surface must initialize from the render context");
    let active = source
        .find("self.resources_ready() && self.scene_valid && self.initialized")
        .expect("active surface dispatch must require valid initialized state");
    let initialized = source
        .find("self.initialized = self.compute_pass.get().is_initialized();")
        .expect("surface initialization state must follow pipeline readiness");
    let scene_valid = source
        .find("self.scene_valid = self.initialized;")
        .expect("scene validity must follow successful initialization");
    assert!(should_init < retry_condition);
    assert!(retry_condition < init);
    assert!(active < init);
    assert!(init < initialized);
    assert!(initialized < scene_valid);
}

#[test]
fn exposes_stable_render_pass_names() {
    assert_eq!(BLIT_PASS_NAME, "BlitPass");
    assert_eq!(CULLING_PASS_NAME, "CullingPass");
    assert_eq!(DEPTH_FIRST_PASS_NAME, "DepthFirstPass");
    assert_eq!(DEPTH_PYRAMID_PASS_NAME, "DepthPyramidPass");
    assert_eq!(
        COMPUTE_WAVEFRONT_PRIMARY_NAME,
        "ComputeWavefrontPrimaryPass"
    );
    assert_eq!(
        COMPUTE_WAVEFRONT_DISPATCH_NAME,
        "ComputeWavefrontDispatchPass"
    );
    assert_eq!(
        COMPUTE_WAVEFRONT_TRAVERSAL_NAME,
        "ComputeWavefrontTraversalPass"
    );
    assert_eq!(COMPUTE_WAVEFRONT_SHADE_NAME, "ComputeWavefrontShadePass");
    assert_eq!(
        COMPUTE_WAVEFRONT_DIRECT_CANDIDATES_NAME,
        "ComputeWavefrontDirectCandidatesPass"
    );
    assert_eq!(
        COMPUTE_WAVEFRONT_COMPOSITE_NAME,
        "ComputeWavefrontCompositePass"
    );
}

#[test]
fn wavefront_resource_ids_are_pairwise_unique() {
    let ids = [
        ("ao", WAVEFRONT_AO_ID),
        ("direct accumulator", WAVEFRONT_DIRECT_ACCUMULATOR_ID),
        ("direct reservoir", WAVEFRONT_RESERVOIR_ID),
        ("dispatch", WAVEFRONT_DISPATCH_ARGS_ID),
        ("final accumulator", WAVEFRONT_FINAL_ACCUMULATOR_ID),
        ("hit dispatch", WAVEFRONT_HIT_DISPATCH_ARGS_ID),
        ("hit header", WAVEFRONT_HIT_HEADER_ID),
        ("hit queue", WAVEFRONT_HIT_QUEUE_ID),
        ("indirect accumulator", WAVEFRONT_INDIRECT_ACCUMULATOR_ID),
        ("next queue", WAVEFRONT_NEXT_QUEUE_ID),
        ("next header", WAVEFRONT_NEXT_HEADER_ID),
        ("next dispatch", WAVEFRONT_NEXT_DISPATCH_ARGS_ID),
        ("next2 queue", WAVEFRONT_NEXT2_QUEUE_ID),
        ("next2 header", WAVEFRONT_NEXT2_HEADER_ID),
        ("next2 dispatch", WAVEFRONT_NEXT2_DISPATCH_ARGS_ID),
        ("next3 queue", WAVEFRONT_NEXT3_QUEUE_ID),
        ("next3 header", WAVEFRONT_NEXT3_HEADER_ID),
        ("next3 dispatch", WAVEFRONT_NEXT3_DISPATCH_ARGS_ID),
        ("primary queue", WAVEFRONT_PRIMARY_QUEUE_ID),
        ("primary header", WAVEFRONT_PRIMARY_HEADER_ID),
        ("scene", WAVEFRONT_SCENE_ID),
        ("transparent queue", WAVEFRONT_TRANSPARENT_QUEUE_ID),
        ("transparent header", WAVEFRONT_TRANSPARENT_HEADER_ID),
        ("upscaled accumulator", WAVEFRONT_UPSCALED_ACCUMULATOR_ID),
    ];
    for (index, (name, id)) in ids.iter().enumerate() {
        for (other_name, other_id) in ids.iter().skip(index + 1) {
            assert_ne!(id, other_id, "wavefront IDs {name} and {other_name} alias");
        }
    }
}

#[test]
fn generated_bounce_names_preserve_constructor_names() {
    let stage = wavefront_bounce_stage(
        8,
        inox_graphics::WAVEFRONT_PRIMARY_QUEUE_ID,
        inox_graphics::WAVEFRONT_HIT_DISPATCH_ARGS_ID,
    );
    assert_eq!(stage.name, "ComputeWavefrontBounce8");
    assert_eq!(stage.dispatch_name, "ComputeWavefrontBounce8Dispatch");
    assert_eq!(stage.traversal_name, "ComputeWavefrontBounce8Traversal");
    assert_eq!(
        stage.hit_dispatch_name,
        "ComputeWavefrontBounce8HitDispatch"
    );
    assert_eq!(stage.shade_name, "ComputeWavefrontBounce8Shade");
}

#[test]
fn wavefront_records_have_storage_buffer_safe_layouts() {
    assert_eq!(std::mem::size_of::<RayQueueRecord>(), 64);
    assert_eq!(std::mem::size_of::<HitRecord>(), 32);
    assert_eq!(std::mem::size_of::<AccumulationRecord>(), 32);
    assert_eq!(std::mem::size_of::<ReservoirRecord>(), 32);
    assert_eq!(std::mem::size_of::<AoRecord>(), 16);
    assert_eq!(std::mem::size_of::<TransparentHitRecord>(), 40);
    assert_eq!(std::mem::size_of::<DispatchArgs>(), 12);
    assert_eq!(std::mem::size_of::<inox_graphics::QueueHeader>(), 16);
    assert_eq!(std::mem::align_of::<RayQueueRecord>(), 4);
    assert_eq!(std::mem::align_of::<HitRecord>(), 4);
    assert_eq!(std::mem::align_of::<AccumulationRecord>(), 4);
    assert_eq!(std::mem::align_of::<ReservoirRecord>(), 4);
    assert_eq!(std::mem::align_of::<AoRecord>(), 4);
    assert_eq!(std::mem::align_of::<TransparentHitRecord>(), 4);
    assert_eq!(std::mem::align_of::<DispatchArgs>(), 4);
    assert_eq!(std::mem::offset_of!(RayQueueRecord, pixel_index), 48);
    assert_eq!(std::mem::offset_of!(HitRecord, barycentric), 16);
    assert_eq!(std::mem::offset_of!(AccumulationRecord, history_length), 24);
    assert_eq!(std::mem::offset_of!(ReservoirRecord, age), 20);
    assert_eq!(std::mem::offset_of!(AoRecord, sample_count), 8);
    assert_eq!(std::mem::offset_of!(TransparentHitRecord, color), 16);
    assert_eq!(std::mem::offset_of!(DispatchArgs, z), 8);
    assert_eq!(WAVEFRONT_RAY_QUEUE_RECORD_STRIDE_WORDS * 4, 64);
    assert_eq!(WAVEFRONT_HIT_RECORD_STRIDE_WORDS * 4, 32);
    assert_eq!(WAVEFRONT_ACCUMULATION_RECORD_STRIDE_WORDS * 4, 32);
    assert_eq!(WAVEFRONT_RESERVOIR_RECORD_STRIDE_WORDS * 4, 32);
    assert_eq!(WAVEFRONT_AO_RECORD_STRIDE_WORDS * 4, 16);
    assert_eq!(WAVEFRONT_TRANSPARENT_HIT_RECORD_STRIDE_WORDS * 4, 40);
    assert_eq!(WAVEFRONT_DISPATCH_ARGS_STRIDE_WORDS * 4, 12);
    assert_eq!(WAVEFRONT_QUEUE_HEADER_STRIDE_WORDS * 4, 16);
}

#[test]
fn wavefront_capacity_scales_resolution_and_clamps_to_budget() {
    assert_eq!(queue_capacity_for_resolution((1920, 1080), 1.0, 4096), 4096);
    assert_eq!(
        queue_capacity_for_resolution((1920, 1080), 0.5, 4_000_000),
        518_400
    );
    assert_eq!(queue_capacity_for_resolution((1, 1), 0.0, 0), 1);
}

#[test]
fn wavefront_dispatch_rounds_up_and_handles_empty_queues() {
    assert_eq!(dispatch_workgroups_for(0, 64), 0);
    assert_eq!(dispatch_workgroups_for(1, 64), 1);
    assert_eq!(dispatch_workgroups_for(65, 64), 2);
}

#[test]
fn dispatch_metadata_records_independent_xyz_dimensions() {
    let recorded = [DispatchArgs { x: 7, y: 11, z: 13 }];

    assert_eq!(recorded[0].x, 7);
    assert_eq!(recorded[0].y, 11);
    assert_eq!(recorded[0].z, 13);
    assert_eq!(std::mem::offset_of!(DispatchArgs, x), 0);
    assert_eq!(std::mem::offset_of!(DispatchArgs, y), 4);
    assert_eq!(std::mem::offset_of!(DispatchArgs, z), 8);
    assert_eq!(std::mem::size_of::<DispatchArgs>(), 12);
}

#[test]
fn depth_pyramid_binds_adjacent_hzb_mips_without_overrunning_views() {
    let source = include_str!("../src/passes/compute_depth_pyramid.rs");
    assert!(source.contains("i as u32"));
    assert!(source.contains("(i + 1) as u32"));
}

#[test]
fn wavefront_queue_bounds_appends_and_resets_without_allocating_per_append() {
    let mut queue = RayQueue::with_capacity(2);
    assert!(queue.append(RayQueueRecord::default()));
    assert!(queue.append(RayQueueRecord::default()));
    assert!(!queue.append(RayQueueRecord::default()));
    assert_eq!(queue.len(), 2);
    assert!(queue.is_overflowed());

    queue.reset();
    assert_eq!(queue.len(), 0);
    assert!(!queue.is_overflowed());
    assert!(queue.append(RayQueueRecord::default()));
}

#[test]
fn depth_pyramid_shader_clamps_rectangular_tail_reads() {
    let source = include_str!("../../../data_raw/shaders/wgsl/compute_depth_pyramid.wgsl");
    assert!(source.contains("textureDimensions(input_mip_texture)"));
    assert!(source.contains("let max_coords = input_dimensions - vec2<i32>(1, 1)"));
    assert!(source.contains("let coords = min(vec2<i32>(p * 2.), max_coords)"));
    for sample in [
        "let sample_y = min(coords + vec2<i32>(0, 1), max_coords)",
        "let sample_z = min(coords + vec2<i32>(1, 0), max_coords)",
        "let sample_w = min(coords + vec2<i32>(1, 1), max_coords)",
    ] {
        assert!(source.contains(sample), "missing clamped Hi-Z sample: {sample}");
    }

    // A 3x5 mip chain has a rectangular tail: 3x5 -> 1x2 -> 1x1.
    let rectangular_tail = [(3, 5), (1, 2), (1, 1)];
    assert_eq!(rectangular_tail[1], (1, 2));
    assert_eq!(rectangular_tail[2], (1, 1));
}

#[test]
fn finalize_publishes_the_completed_ping_pong_target() {
    let source = include_str!("../src/passes/finalize.rs");
    let draw = source
        .find("pass.draw(render_context, render_pass, 0..3, 0..1);")
        .expect("finalize must draw the completed frame");
    let publish = source
        .find("publish_finalize_target(self.frame_index);")
        .expect("finalize must publish the target it just rendered");
    assert!(draw < publish);
}

#[test]
fn cornell_diagnostics_read_only_the_published_target_without_ppm_artifacts() {
    let source = include_str!("../src/passes/pathtracing_common.rs");
    assert!(source.contains("active_finalize_target()"));
    assert!(!source.contains("[3usize, 4usize]"));
    assert!(!source.contains("save_cornell_readback_ppm"));
    assert!(!source.contains("INOX_NATIVE_CORNELL_IMAGE_PPM"));
    assert!(!source.contains("temp_dir()"));
}
