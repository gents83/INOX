#![cfg(all(not(target_arch = "wasm32")))]

pub fn required_gpu_features() -> wgpu::Features {
    wgpu::Features::default()
        | wgpu::Features::POLYGON_MODE_LINE
        | wgpu::Features::INDIRECT_FIRST_INSTANCE
        | wgpu::Features::SPIRV_SHADER_PASSTHROUGH
        | wgpu::Features::MULTI_DRAW_INDIRECT
        | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT
}

pub fn is_indirect_mode_enabled() -> bool {
    true
}
