#![cfg(target_arch = "wasm32")]

pub fn required_gpu_features() -> wgpu::Features {
    wgpu::Features::default() | wgpu::Features::POLYGON_MODE_LINE
}

pub fn is_indirect_mode_enabled() -> bool {
    false
}
