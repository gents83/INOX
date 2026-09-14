use inox_graphics::{
    BLIT_PASS_NAME, CULLING_PASS_NAME, DEPTH_FIRST_PASS_NAME, DEPTH_PYRAMID_PASS_NAME,
};

#[test]
fn exposes_stable_render_pass_names() {
    assert_eq!(BLIT_PASS_NAME, "BlitPass");
    assert_eq!(CULLING_PASS_NAME, "CullingPass");
    assert_eq!(DEPTH_FIRST_PASS_NAME, "DepthFirstPass");
    assert_eq!(DEPTH_PYRAMID_PASS_NAME, "DepthPyramidPass");
}
