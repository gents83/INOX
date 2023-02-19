#![cfg(target_arch = "wasm32")]

pub fn required_gpu_features() -> wgpu::Features {
    wgpu::Features::default() | wgpu::Features::POLYGON_MODE_LINE | wgpu::Features::CLEAR_TEXTURE
}

pub fn platform_limits() -> wgpu::Limits {
    wgpu::Limits::default()
}
