use inox_render::{
    platform::{
        negotiate_gpu_requirements, optional_gpu_features, platform_limits,
        platform_limits_from_adapter_limits, required_gpu_features, RenderCapabilities,
        PORTABLE_MAX_BINDINGS_PER_SHADER_STAGE, PORTABLE_MAX_STORAGE_BUFFERS_PER_SHADER_STAGE,
    },
    BindingFlags,
};

#[test]
fn gpu_read_binding_keeps_storage_without_cpu_upload_usage() {
    let usage: wgpu::BufferUsages = (BindingFlags::Storage | BindingFlags::GPURead).into();
    assert!(usage.contains(wgpu::BufferUsages::STORAGE));
    assert!(!usage.contains(wgpu::BufferUsages::COPY_DST));
}

#[test]
fn platform_limits_clamps_storage_buffer_capacity_for_all_stages() {
    let requested_limits = platform_limits();
    let adapter_limits = wgpu::Limits {
        max_storage_buffers_per_shader_stage: 16,
        max_storage_buffers_in_vertex_stage: 12,
        max_storage_buffers_in_fragment_stage: 14,
        max_binding_array_elements_per_shader_stage: 256,
        ..Default::default()
    };

    let limits = platform_limits_from_adapter_limits(requested_limits.clone(), &adapter_limits);

    assert_eq!(
        limits.max_storage_buffers_per_shader_stage,
        requested_limits
            .max_storage_buffers_per_shader_stage
            .min(adapter_limits.max_storage_buffers_per_shader_stage)
    );
    assert_eq!(
        limits.max_storage_buffers_in_vertex_stage,
        requested_limits
            .max_storage_buffers_in_vertex_stage
            .min(adapter_limits.max_storage_buffers_in_vertex_stage)
    );
    assert_eq!(
        limits.max_storage_buffers_in_fragment_stage,
        requested_limits
            .max_storage_buffers_in_fragment_stage
            .min(adapter_limits.max_storage_buffers_in_fragment_stage)
    );
    assert_eq!(
        limits.max_binding_array_elements_per_shader_stage,
        requested_limits.max_binding_array_elements_per_shader_stage
    );
}

#[test]
fn render_capabilities_expose_a_conservative_portable_binding_budget() {
    let limits = wgpu::Limits {
        max_storage_buffers_per_shader_stage: 32,
        max_bindings_per_bind_group: 100,
        ..Default::default()
    };
    let capabilities = RenderCapabilities::from_adapter(&limits, wgpu::Features::empty());

    assert_eq!(
        capabilities.max_storage_buffers_per_shader_stage,
        PORTABLE_MAX_STORAGE_BUFFERS_PER_SHADER_STAGE
    );
    assert_eq!(
        capabilities.max_bindings_per_shader_stage,
        PORTABLE_MAX_BINDINGS_PER_SHADER_STAGE
    );
    assert!(!capabilities.texture_binding_arrays);
    assert!(!capabilities.buffer_binding_arrays);
}

#[test]
fn render_capabilities_keep_discrete_adapter_class_explicit() {
    let limits = wgpu::Limits {
        max_storage_buffers_per_shader_stage: 32,
        max_bindings_per_bind_group: 100,
        ..Default::default()
    };
    let discrete =
        RenderCapabilities::from_adapter_with_class(&limits, wgpu::Features::empty(), true);
    let integrated =
        RenderCapabilities::from_adapter_with_class(&limits, wgpu::Features::empty(), false);
    assert!(discrete.discrete_adapter);
    assert!(!integrated.discrete_adapter);
}

#[test]
fn render_capabilities_keep_supported_optional_features_without_changing_baseline_budget() {
    let limits = wgpu::Limits {
        max_storage_buffers_per_shader_stage: 12,
        max_bindings_per_bind_group: 32,
        ..Default::default()
    };
    let features = wgpu::Features::TEXTURE_BINDING_ARRAY
        | wgpu::Features::BUFFER_BINDING_ARRAY
        | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT;
    let capabilities = RenderCapabilities::from_adapter(&limits, features);

    assert!(capabilities.texture_binding_arrays);
    assert!(capabilities.buffer_binding_arrays);
    assert!(capabilities.multi_draw_indirect_count);
    assert_eq!(
        capabilities.max_storage_buffers_per_shader_stage,
        PORTABLE_MAX_STORAGE_BUFFERS_PER_SHADER_STAGE
    );
}

#[test]
fn platform_limits_clamps_binding_array_capacity_to_the_adapter() {
    let requested_limits = platform_limits();
    let adapter_limits = wgpu::Limits {
        max_binding_array_elements_per_shader_stage: 0,
        ..Default::default()
    };

    let limits = platform_limits_from_adapter_limits(requested_limits, &adapter_limits);

    assert_eq!(limits.max_binding_array_elements_per_shader_stage, 0);
}

#[test]
fn negotiation_uses_baseline_features_when_adapter_has_no_optional_features() {
    let requirements =
        negotiate_gpu_requirements(required_gpu_features(), &wgpu::Limits::default())
            .expect("baseline adapter should be accepted");

    assert_eq!(requirements.required_features, required_gpu_features());
    assert!(!requirements.capabilities.wireframe);
    assert!(!requirements.capabilities.primitive_index);
    assert!(!requirements.capabilities.multi_draw_indirect_count);
    assert!(!requirements.capabilities.timestamp_queries);
}

#[test]
fn negotiation_requests_only_supported_optional_feature_subset() {
    let supported = required_gpu_features()
        | wgpu::Features::POLYGON_MODE_LINE
        | wgpu::Features::TEXTURE_BINDING_ARRAY;
    let requirements = negotiate_gpu_requirements(supported, &wgpu::Limits::default())
        .expect("baseline plus supported optionals should be accepted");

    assert!(requirements
        .required_features
        .contains(wgpu::Features::POLYGON_MODE_LINE));
    assert!(requirements
        .required_features
        .contains(wgpu::Features::TEXTURE_BINDING_ARRAY));
    assert!(!requirements
        .required_features
        .contains(wgpu::Features::PRIMITIVE_INDEX));
    assert_eq!(
        requirements.required_features & !required_gpu_features(),
        supported & optional_gpu_features()
    );
}

#[test]
fn negotiation_accepts_downlevel_storage_limit_and_disables_wavefront_layout() {
    let limits = wgpu::Limits {
        max_storage_buffers_per_shader_stage: PORTABLE_MAX_STORAGE_BUFFERS_PER_SHADER_STAGE - 1,
        ..Default::default()
    };

    let requirements = negotiate_gpu_requirements(required_gpu_features(), &limits)
        .expect("storage-buffer count is a layout capability, not a baseline requirement");
    assert_eq!(
        requirements
            .capabilities
            .max_storage_buffers_per_shader_stage,
        PORTABLE_MAX_STORAGE_BUFFERS_PER_SHADER_STAGE - 1
    );
    assert!(!requirements.capabilities.wavefront_layout_supported);
}
