use inox_graphics::{dispatch_workgroups_for, queue_capacity_for_resolution};
use inox_render::{FrameDimensions, RenderScalePolicy};

#[test]
fn non_default_dimensions_drive_matching_queue_and_dispatch_sizes() {
    let dimensions = FrameDimensions::new(321, 197, 1.0).unwrap();
    let render = dimensions.render_dimensions();
    assert_eq!(render, (321, 197));
    assert_eq!(
        queue_capacity_for_resolution(
            render.0,
            render.1,
            16,
            1,
            inox_render::QueueCapacityLimits {
                fallback_capacity: u32::MAX,
                ..Default::default()
            },
        )
        .unwrap(),
        321 * 197
    );
    assert_eq!(
        [
            dispatch_workgroups_for(render.0, 8),
            dispatch_workgroups_for(render.1, 8),
        ],
        dimensions.dispatch
    );
}

#[test]
fn zero_resize_uses_clamped_one_pixel_contract() {
    let dimensions = FrameDimensions::new(0, 0, 1.0).unwrap();
    let render = dimensions.render_dimensions();
    assert_eq!(render, (1, 1));
    assert_eq!(
        queue_capacity_for_resolution(
            render.0,
            render.1,
            16,
            1,
            inox_render::QueueCapacityLimits::default(),
        )
        .unwrap(),
        1
    );
    assert_eq!(
        [
            dispatch_workgroups_for(render.0, 8),
            dispatch_workgroups_for(render.1, 8),
        ],
        [1, 1]
    );
}

#[test]
fn scaled_render_dispatch_keeps_full_resolution_stride() {
    let dimensions =
        FrameDimensions::with_policy(321, 197, 0.5, RenderScalePolicy::ScaleRenderExtent).unwrap();
    assert_eq!(dimensions.render_dimensions(), (161, 99));
    assert_eq!(dimensions.storage_stride, 321);
    assert_eq!(dimensions.dispatch, [21, 13]);
}

#[test]
fn resized_hiz_extent_uses_minimum_and_derived_mip_counts() {
    assert_eq!(FrameDimensions::new(0, 0, 1.0).unwrap().hiz_mip_count(), 1);
    assert_eq!(FrameDimensions::new(17, 9, 1.0).unwrap().hiz_mip_count(), 5);
}
