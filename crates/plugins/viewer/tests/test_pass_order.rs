use std::collections::HashSet;

use inox_graphics::{
    wavefront_auxiliary_bindings, wavefront_bounce_stage, wavefront_primary_bindings,
    wavefront_primary_shade_bindings, WAVEFRONT_AO_ID, WAVEFRONT_HIT_DISPATCH_ARGS_ID,
    WAVEFRONT_HIT_HEADER_ID, WAVEFRONT_HIT_QUEUE_ID, WAVEFRONT_PRIMARY_QUEUE_ID,
    WAVEFRONT_TRANSPARENT_HEADER_ID, WAVEFRONT_TRANSPARENT_QUEUE_ID,
};
use inox_viewer::viewer::production_pass_metadata;

fn active_pass_names() -> Vec<String> {
    production_pass_metadata()
        .into_iter()
        .filter(|pass| pass.enabled)
        .map(|pass| pass.name)
        .collect()
}

#[test]
fn production_metadata_has_unique_entries_and_complete_queue_chain() {
    let metadata = production_pass_metadata();
    let active = active_pass_names();
    let names: HashSet<_> = active.iter().map(String::as_str).collect();
    assert_eq!(names.len(), active.len());
    assert_eq!(active.len(), 63);

    let fixed_prefix = [
        "DepthFirstPass",
        "DepthPyramidPass",
        "ComputeInstancesPass",
        "CullingPass",
        "CommandsPass",
        "VisibilityBufferPass",
        "ComputeWavefrontSurfacePass",
        "ComputeWavefrontTransparencyPass",
        "ComputeWavefrontAoResolvePass",
        "ComputeWavefrontDirectCandidatesPass",
        "ComputeWavefrontVisibilityPass",
        "ComputeWavefrontPrimaryPass",
        "ComputeWavefrontDispatchPass",
        "ComputeWavefrontTraversalPass",
        "ComputeWavefrontHitDispatchPass",
        "ComputeWavefrontShadePass",
    ];
    assert_eq!(&active[..fixed_prefix.len()], fixed_prefix);

    let mut position = fixed_prefix.len();
    let mut source_queue = WAVEFRONT_PRIMARY_QUEUE_ID;
    let mut source_dispatch = WAVEFRONT_HIT_DISPATCH_ARGS_ID;
    for bounce in 1..=8 {
        let stage = wavefront_bounce_stage(bounce, source_queue, source_dispatch);
        let expected = [
            stage.name.as_str(),
            stage.dispatch_name.as_str(),
            stage.traversal_name.as_str(),
            stage.hit_dispatch_name.as_str(),
            stage.shade_name.as_str(),
        ];
        assert_eq!(&active[position..position + expected.len()], expected);
        position += expected.len();
        source_queue = stage.bindings.destination_ray_queue;
        source_dispatch = WAVEFRONT_HIT_DISPATCH_ARGS_ID;
    }
    assert_eq!(
        active[position..]
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "ComputeWavefrontCompositePass",
            "ComputeWavefrontTransparencyResolvePass",
            "ComputeWavefrontDenoiseTemporalPass",
            "ComputeWavefrontDenoiseBilateralPass",
            "ComputeWavefrontUpscalePass",
            "FinalizePass",
            "BlitPass",
        ]
    );

    let disabled = metadata
        .into_iter()
        .filter(|pass| !pass.enabled)
        .map(|pass| pass.name)
        .collect::<HashSet<_>>();
    assert_eq!(
        disabled,
        [
            "ComputeGBufferPass",
            "ComputeShadowAOPass",
            "ComputeDiffuseLightingPass",
            "ComputeSpecularLightingPass",
            "ComputeIndirectSetupPass",
            "ComputeIndirectBouncePass",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
}

#[test]
fn wavefront_graph_contract_connects_queues_headers_and_dispatch_resources() {
    let primary = wavefront_primary_bindings();
    assert_eq!(primary.ray_queue, WAVEFRONT_PRIMARY_QUEUE_ID);
    assert_eq!(
        primary.ray_header,
        inox_graphics::WAVEFRONT_PRIMARY_HEADER_ID
    );
    assert_eq!(
        primary.dispatch_args,
        inox_graphics::WAVEFRONT_DISPATCH_ARGS_ID
    );
    let primary_shade = wavefront_primary_shade_bindings();
    assert_eq!(primary_shade.ray_queue, primary.ray_queue);
    assert_eq!(primary_shade.ray_header, primary.ray_header);
    assert_eq!(primary_shade.hit_queue, primary.hit_queue);
    assert_eq!(primary_shade.hit_header, primary.hit_header);
    assert_eq!(primary_shade.dispatch_args, WAVEFRONT_HIT_DISPATCH_ARGS_ID);
    assert_eq!(primary.hit_queue, WAVEFRONT_HIT_QUEUE_ID);
    assert_eq!(primary.hit_header, WAVEFRONT_HIT_HEADER_ID);

    let mut source_queue = WAVEFRONT_PRIMARY_QUEUE_ID;
    let mut source_dispatch = WAVEFRONT_HIT_DISPATCH_ARGS_ID;
    for index in 1..=8 {
        let stage = wavefront_bounce_stage(index, source_queue, source_dispatch);
        assert_eq!(stage.bindings.source_ray_queue, source_queue);
        assert_eq!(stage.bindings.source_hit_queue, WAVEFRONT_HIT_QUEUE_ID);
        assert_eq!(stage.bindings.source_hit_header, WAVEFRONT_HIT_HEADER_ID);
        assert_eq!(stage.bindings.dispatch_args, source_dispatch);
        assert_eq!(
            stage.traversal_bindings.ray_queue,
            stage.bindings.destination_ray_queue
        );
        assert_eq!(
            stage.traversal_bindings.ray_header,
            stage.bindings.destination_ray_header
        );
        assert_eq!(
            stage.traversal_bindings.dispatch_args,
            stage.destination_dispatch_args
        );
        assert_eq!(stage.traversal_bindings.hit_queue, WAVEFRONT_HIT_QUEUE_ID);
        assert_eq!(stage.traversal_bindings.hit_header, WAVEFRONT_HIT_HEADER_ID);
        assert_eq!(
            stage.shade_bindings.ray_queue,
            stage.traversal_bindings.ray_queue
        );
        assert_eq!(
            stage.shade_bindings.ray_header,
            stage.traversal_bindings.ray_header
        );
        assert_eq!(stage.shade_bindings.hit_queue, WAVEFRONT_HIT_QUEUE_ID);
        assert_eq!(stage.shade_bindings.hit_header, WAVEFRONT_HIT_HEADER_ID);
        assert_eq!(
            stage.shade_bindings.dispatch_args,
            WAVEFRONT_HIT_DISPATCH_ARGS_ID
        );
        source_queue = stage.bindings.destination_ray_queue;
        source_dispatch = WAVEFRONT_HIT_DISPATCH_ARGS_ID;
    }

    let auxiliary = wavefront_auxiliary_bindings();
    assert_eq!(auxiliary.transparent_queue, WAVEFRONT_TRANSPARENT_QUEUE_ID);
    assert_eq!(
        auxiliary.transparent_header,
        WAVEFRONT_TRANSPARENT_HEADER_ID
    );
    assert_eq!(auxiliary.ao_state, WAVEFRONT_AO_ID);
}

#[test]
fn production_pass_contract_keeps_dependencies_and_reconstruction_tail_ordered() {
    let active = active_pass_names();
    let position = |name: &str| active.iter().position(|pass| pass == name).unwrap();
    assert!(position("DepthFirstPass") < position("VisibilityBufferPass"));
    assert!(
        position("ComputeWavefrontSurfacePass") < position("ComputeWavefrontDirectCandidatesPass")
    );
    assert!(
        position("ComputeWavefrontDirectCandidatesPass")
            < position("ComputeWavefrontVisibilityPass")
    );
    assert!(position("ComputeWavefrontVisibilityPass") < position("ComputeWavefrontCompositePass"));
    assert!(position("ComputeWavefrontCompositePass") < position("FinalizePass"));
    assert!(position("ComputeWavefrontUpscalePass") < position("FinalizePass"));
    assert!(
        position("ComputeWavefrontTransparencyPass")
            < position("ComputeWavefrontTransparencyResolvePass")
    );
    assert!(position("ComputeWavefrontAoResolvePass") < position("ComputeWavefrontCompositePass"));
}
