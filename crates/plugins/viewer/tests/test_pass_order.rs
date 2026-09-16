use std::collections::HashSet;

use inox_viewer::viewer::PRODUCTION_PASS_ORDER;

#[test]
fn production_pass_order_has_unique_entries_and_required_stages() {
    let names: HashSet<_> = PRODUCTION_PASS_ORDER.iter().copied().collect();
    assert_eq!(names.len(), PRODUCTION_PASS_ORDER.len());
    for required in [
        "DepthFirstPass",
        "VisibilityBufferPass",
        "ComputeWavefrontSurfacePass",
        "ComputeWavefrontDirectCandidatesPass",
        "ComputeWavefrontVisibilityPass",
        "ComputeWavefrontCompositePass",
        "ComputeWavefrontDenoiseTemporalPass",
        "ComputeWavefrontUpscalePass",
        "FinalizePass",
    ] {
        assert!(
            names.contains(required),
            "missing production pass {required}"
        );
    }
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
