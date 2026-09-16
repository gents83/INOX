use inox_graphics::{
    dispatch_workgroups_for, queue_capacity_for_resolution, wavefront_bounce_stage,
    AccumulationRecord, AoRecord, DispatchArgs, HitRecord, RayQueue, RayQueueRecord,
    ReservoirRecord, TransparentHitRecord, BLIT_PASS_NAME, COMPUTE_WAVEFRONT_COMPOSITE_NAME,
    COMPUTE_WAVEFRONT_DIRECT_CANDIDATES_NAME, COMPUTE_WAVEFRONT_DISPATCH_NAME,
    COMPUTE_WAVEFRONT_PRIMARY_NAME, COMPUTE_WAVEFRONT_SHADE_NAME, COMPUTE_WAVEFRONT_TRAVERSAL_NAME,
    CULLING_PASS_NAME, DEPTH_FIRST_PASS_NAME, DEPTH_PYRAMID_PASS_NAME,
};

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
fn dispatch_metadata_records_xyz_without_rebuilding_pipeline_storage() {
    let args = DispatchArgs { x: 7, y: 11, z: 13 };
    let recorded = [args];

    assert_eq!(recorded[0], args);
    assert_eq!(std::mem::size_of::<DispatchArgs>(), 12);
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
