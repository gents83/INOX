use inox_platform::{PlatformType, PLATFORM_TYPE_PC};
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(target_os = "windows")]
pub use pc::*;
#[cfg(target_os = "windows")]
pub mod pc;

pub const WGPU_FIXED_ALIGNMENT: u64 = 16; // 4 bytes is min alignment for wgpu

pub fn shader_preprocessor_defs<const PLATFORM_TYPE: PlatformType>() -> Vec<String> {
    if PLATFORM_TYPE == PLATFORM_TYPE_PC {
        vec![
            "FEATURES_TEXTURE_BINDING_ARRAY".to_string(),
            "FEATURES_MULTISAMPLING".to_string(),
        ]
    } else {
        vec![]
    }
}

pub fn has_wireframe_support() -> bool {
    required_gpu_features().contains(wgpu::Features::POLYGON_MODE_LINE)
}
pub fn has_primitive_index_support() -> bool {
    required_gpu_features().contains(wgpu::Features::SHADER_PRIMITIVE_INDEX)
}
pub fn is_indirect_mode_enabled() -> bool {
    required_gpu_features().contains(wgpu::Features::MULTI_DRAW_INDIRECT)
}
pub fn has_timestamp_queries() -> bool {
    required_gpu_features().contains(wgpu::Features::TIMESTAMP_QUERY)
}
