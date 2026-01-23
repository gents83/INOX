pub fn required_gpu_features() -> wgpu::Features {
    wgpu::Features::default()
        | wgpu::Features::POLYGON_MODE_LINE
        | wgpu::Features::INDIRECT_FIRST_INSTANCE
        | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS
        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
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
        unsafe {
            std::env::set_var("WGPU_POWER_PREF", "high");
        }
    }
}

pub fn has_multisampling_support() -> bool {
    true
}
