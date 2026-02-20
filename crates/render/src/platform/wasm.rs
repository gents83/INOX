pub fn required_gpu_features() -> wgpu::Features {
    wgpu::Features::default()
}

pub fn platform_limits() -> wgpu::Limits {
    wgpu::Limits::downlevel_webgl2_defaults()
}

pub fn setup_env() {
    //...
}

pub fn has_multisampling_support() -> bool {
    false
}
