use inox_render::platform::{platform_limits, platform_limits_from_adapter_limits};

#[test]
fn platform_limits_preserves_adapter_storage_buffer_capacity_for_all_stages() {
    let requested_limits = platform_limits();
    let mut adapter_limits = wgpu::Limits::default();
    adapter_limits.max_storage_buffers_per_shader_stage = 16;
    adapter_limits.max_storage_buffers_in_vertex_stage = 12;
    adapter_limits.max_storage_buffers_in_fragment_stage = 14;

    let limits = platform_limits_from_adapter_limits(requested_limits.clone(), &adapter_limits);

    assert_eq!(limits.max_storage_buffers_per_shader_stage, 16);
    assert_eq!(limits.max_storage_buffers_in_vertex_stage, 12);
    assert_eq!(limits.max_storage_buffers_in_fragment_stage, 14);
    assert_eq!(
        limits.max_binding_array_elements_per_shader_stage,
        requested_limits.max_binding_array_elements_per_shader_stage
    );
}
