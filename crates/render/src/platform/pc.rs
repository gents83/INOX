#![cfg(not(target_arch = "wasm32"))]

pub fn required_gpu_features() -> wgpu::Features {
    wgpu::Features::default()
        | wgpu::Features::POLYGON_MODE_LINE
        | wgpu::Features::INDIRECT_FIRST_INSTANCE
        | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS
        | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT
        | wgpu::Features::TEXTURE_BINDING_ARRAY
        | wgpu::Features::BUFFER_BINDING_ARRAY
        | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY
        | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
        | wgpu::Features::STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING
        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
        | wgpu::Features::SHADER_PRIMITIVE_INDEX
        | wgpu::Features::PIPELINE_STATISTICS_QUERY
        | wgpu::Features::TIMESTAMP_QUERY
        | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES
        | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS
        | wgpu::Features::PUSH_CONSTANTS
        | wgpu::Features::DEPTH32FLOAT_STENCIL8
        | wgpu::Features::VERTEX_WRITABLE_STORAGE
        | wgpu::Features::CLEAR_TEXTURE
}

pub fn platform_limits() -> wgpu::Limits {
    wgpu::Limits {
        max_binding_array_elements_per_shader_stage: 256,
        ..Default::default()
    }
}

pub fn setup_env() {
    let wgpu_power_pref = std::env::var("WGPU_POWER_PREF");
    if wgpu_power_pref.is_err() {
        std::env::set_var("WGPU_POWER_PREF", "high");
    }
}

pub fn has_multisampling_support() -> bool {
    true
}
