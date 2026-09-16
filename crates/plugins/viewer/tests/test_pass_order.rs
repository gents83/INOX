use std::collections::HashSet;

use inox_viewer::viewer::PRODUCTION_PASS_ORDER;

#[test]
fn production_pass_order_has_unique_entries_and_complete_queue_chain() {
    let names: HashSet<_> = PRODUCTION_PASS_ORDER.iter().copied().collect();
    assert_eq!(names.len(), PRODUCTION_PASS_ORDER.len());
    assert_eq!(PRODUCTION_PASS_ORDER.len(), 63);

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
    assert_eq!(&PRODUCTION_PASS_ORDER[..fixed_prefix.len()], fixed_prefix);

    let mut position = fixed_prefix.len();
    for bounce in 1..=8 {
        for suffix in ["", "Dispatch", "Traversal", "HitDispatch", "Shade"] {
            let expected = format!("ComputeWavefrontBounce{bounce}{suffix}");
            assert_eq!(PRODUCTION_PASS_ORDER[position], expected);
            assert!(names.contains(PRODUCTION_PASS_ORDER[position]));
            position += 1;
        }
    }
    assert_eq!(
        PRODUCTION_PASS_ORDER[position..],
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
}

#[test]
fn production_pass_contract_keeps_visibility_before_lighting_and_composite_before_finalize() {
    let position = |name: &str| {
        PRODUCTION_PASS_ORDER
            .iter()
            .position(|pass| *pass == name)
            .unwrap()
    };
    assert!(position("DepthFirstPass") < position("VisibilityBufferPass"));
    assert!(
        position("ComputeWavefrontSurfacePass") < position("ComputeWavefrontDirectCandidatesPass")
    );
    assert!(
        position("ComputeWavefrontDirectCandidatesPass")
            < position("ComputeWavefrontCompositePass")
    );
    assert!(position("ComputeWavefrontCompositePass") < position("FinalizePass"));
    assert!(position("ComputeWavefrontUpscalePass") < position("FinalizePass"));
}
