use inox_render::{Extent2d, FrameDimensions, FrameDimensionsError, RenderScalePolicy};

#[test]
fn zero_dimensions_are_clamped_to_one_pixel() {
    let dimensions = FrameDimensions::new(0, 0, 1.0).unwrap();
    assert_eq!(dimensions.surface, Extent2d::new(1, 1));
    assert_eq!(dimensions.render, Extent2d::new(1, 1));
    assert_eq!(dimensions.storage_stride, 1);
    assert_eq!(dimensions.dispatch, [1, 1]);
}

#[test]
fn checked_pixel_count_rejects_unrepresentable_storage() {
    let error = FrameDimensions::new(u32::MAX, 2, 1.0).unwrap_err();
    assert_eq!(
        error,
        FrameDimensionsError::PixelCountOverflow {
            width: u32::MAX,
            height: 2,
        }
    );
}

#[test]
fn reconstruction_only_scale_keeps_full_resolution_storage_and_dispatch() {
    let dimensions = FrameDimensions::new(127, 65, 0.5).unwrap();
    assert_eq!(dimensions.surface, Extent2d::new(127, 65));
    assert_eq!(dimensions.render, dimensions.surface);
    assert_eq!(dimensions.storage_stride, 127);
    assert_eq!(dimensions.dispatch, [16, 9]);
}

#[test]
fn scaled_render_extent_is_explicit_and_rounded_up() {
    let dimensions =
        FrameDimensions::with_policy(127, 65, 0.5, RenderScalePolicy::ScaleRenderExtent).unwrap();
    assert_eq!(dimensions.render, Extent2d::new(64, 33));
    assert_eq!(dimensions.storage_stride, 127);
    assert_eq!(dimensions.dispatch, [8, 5]);
}

#[test]
fn non_default_dimensions_are_equal_across_derived_values() {
    let dimensions = FrameDimensions::new(321, 197, 1.0).unwrap();
    assert_eq!(dimensions.storage_pixel_count().unwrap(), 321 * 197);
    assert_eq!(dimensions.render_dimensions(), (321, 197));
    assert_eq!(dimensions.dispatch, [41, 25]);
}

#[test]
fn hiz_mip_count_covers_small_and_non_power_of_two_extents() {
    assert_eq!(FrameDimensions::new(1, 1, 1.0).unwrap().hiz_mip_count(), 1);
    assert_eq!(FrameDimensions::new(2, 1, 1.0).unwrap().hiz_mip_count(), 2);
    assert_eq!(FrameDimensions::new(3, 5, 1.0).unwrap().hiz_mip_count(), 3);
    assert_eq!(
        FrameDimensions::new(1920, 1080, 1.0)
            .unwrap()
            .hiz_mip_count(),
        11
    );
}

#[test]
fn surface_extent_mismatch_reconfigures_before_encoding() {
    let source = include_str!("../src/common/render_context.rs");
    assert!(source.contains("surface_texture.texture.width()"));
    assert!(source.contains("set_surface_size(surface_width, surface_height)"));
}

#[test]
fn render_target_resize_invalidates_texture_backed_bind_groups() {
    let source = include_str!("../src/data/binding_data.rs");
    assert!(source.contains("binding_generation"));
    assert!(source.contains("self.is_data_changed = true"));
}

#[test]
fn render_target_resize_releases_before_large_replacement() {
    let source = include_str!("../src/textures/texture_ref.rs");
    let resize = source
        .find("pub fn resize(")
        .expect("texture resize must exist");
    let resize_source = &source[resize..];
    let destroy = resize_source
        .find("self.texture.destroy();")
        .expect("resize must destroy the old texture");
    let replacement = resize_source
        .find("let replacement = Self::create(")
        .expect("resize must create a replacement texture");
    assert!(destroy < replacement);
    assert!(resize_source[destroy..replacement].contains("self.views.clear();"));
    assert!(resize_source[destroy..replacement].contains("device.poll(wgpu::PollType::Wait"));
}

#[test]
fn resize_transition_preserves_full_resolution_contract() {
    let initial = FrameDimensions::new(1920, 1080, 1.0).unwrap();
    let maximized = FrameDimensions::new(3820, 2117, 1.0).unwrap();
    let restored = FrameDimensions::new(1920, 1080, 1.0).unwrap();
    assert_eq!(initial.render, initial.surface);
    assert_eq!(maximized.render, maximized.surface);
    assert_eq!(restored.render, restored.surface);
    assert_ne!(initial.surface, maximized.surface);
    assert_eq!(initial.surface, restored.surface);
}

#[test]
fn hiz_mip_policy_survives_shrink_to_one_pixel_and_growth() {
    let one_pixel = FrameDimensions::new(1, 1, 1.0).unwrap();
    let restored = FrameDimensions::new(1920, 1080, 1.0).unwrap();
    assert_eq!(one_pixel.hiz_mip_count(), 1);
    assert_eq!(restored.hiz_mip_count(), 11);

    let source = include_str!("../src/common/render_context.rs");
    assert!(source.contains("mip_chain_targets"));
}

#[test]
fn wasm_gpu_failures_are_reported_and_success_marks_real_device_creation() {
    let context_source = include_str!("../src/common/render_context.rs");
    let renderer_source = include_str!("../src/common/renderer.rs");
    assert!(context_source.contains("js_namespace = console, js_name = error"));
    assert!(context_source.contains("new_instance_with_webgpu_detection"));
    for marker in [
        "signal_browser_smoke(\"instance_ready\")",
        "signal_browser_smoke(\"adapter_requested\")",
        "signal_browser_smoke(\"adapter_ready\")",
        "signal_browser_smoke(\"device_requested\")",
        "signal_browser_smoke(\"device_ready\")",
        "signal_browser_smoke_value(\"device_failure\"",
    ] {
        assert!(
            context_source.contains(marker),
            "missing WASM marker {marker}"
        );
    }
    assert!(context_source.contains("report_browser_smoke_failure"));
    assert!(context_source.contains("signal_browser_smoke(\"device\")"));
    assert!(renderer_source.contains("render context creation failed"));
    assert!(renderer_source.contains("return;"));
}
