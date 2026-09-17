use inox_render::{ConstantData, DENOISER_TEMPORAL_VARIANCE, UPSCALER_NATIVE};

#[test]
fn constant_data_matches_wgsl_uniform_alignment() {
    assert_eq!(std::mem::size_of::<ConstantData>(), 416);
}

#[test]
fn quality_defaults_select_portable_temporal_denoising_and_native_resolution() {
    let data = ConstantData::default();
    assert_eq!(data.denoiser_mode(), DENOISER_TEMPORAL_VARIANCE);
    assert_eq!(data.upscaler_mode(), UPSCALER_NATIVE);
    assert_eq!(data.render_scale(), 1.0);
    assert_eq!(data.max_transparency_layers(), 4);
    assert_eq!(data.num_bounces(), 2);
}
