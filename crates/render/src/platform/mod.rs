use inox_platform::{PlatformType, PLATFORM_TYPE_PC};

/// Conservative limits used by the portable wavefront layout.
///
/// The renderer intentionally stays below these values even when a desktop
/// adapter exposes larger limits, so the same pipelines can run on WebGPU and
/// mobile adapters.
pub const PORTABLE_MAX_BINDINGS_PER_SHADER_STAGE: u32 = 8;
pub const PORTABLE_MAX_STORAGE_BUFFERS_PER_SHADER_STAGE: u32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderCapabilities {
    pub max_storage_buffers_per_shader_stage: u32,
    pub max_bindings_per_shader_stage: u32,
    pub wireframe: bool,
    pub primitive_index: bool,
    pub timestamp_queries: bool,
    pub pipeline_statistics_queries: bool,
    pub texture_binding_arrays: bool,
    pub buffer_binding_arrays: bool,
    pub multi_draw_indirect_count: bool,
}

impl RenderCapabilities {
    pub fn from_adapter(limits: &wgpu::Limits, features: wgpu::Features) -> Self {
        Self {
            max_storage_buffers_per_shader_stage: PORTABLE_MAX_STORAGE_BUFFERS_PER_SHADER_STAGE
                .min(limits.max_storage_buffers_per_shader_stage),
            max_bindings_per_shader_stage: PORTABLE_MAX_BINDINGS_PER_SHADER_STAGE
                .min(limits.max_bindings_per_bind_group),
            wireframe: features.contains(wgpu::Features::POLYGON_MODE_LINE),
            primitive_index: features.contains(wgpu::Features::PRIMITIVE_INDEX),
            timestamp_queries: features.contains(wgpu::Features::TIMESTAMP_QUERY),
            pipeline_statistics_queries: features
                .contains(wgpu::Features::PIPELINE_STATISTICS_QUERY),
            texture_binding_arrays: features.contains(wgpu::Features::TEXTURE_BINDING_ARRAY),
            buffer_binding_arrays: features.contains(wgpu::Features::BUFFER_BINDING_ARRAY),
            multi_draw_indirect_count: features.contains(wgpu::Features::MULTI_DRAW_INDIRECT_COUNT),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuRequirementsError {
    MissingRequiredFeatures(wgpu::Features),
    InsufficientLimit {
        name: &'static str,
        required: u32,
        available: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiatedGpuRequirements {
    pub required_features: wgpu::Features,
    pub required_limits: wgpu::Limits,
    pub capabilities: RenderCapabilities,
}

/// Features used only to select optional rendering paths. These must never be
/// included in the baseline device request.
pub fn optional_gpu_features() -> wgpu::Features {
    wgpu::Features::POLYGON_MODE_LINE
        | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT
        | wgpu::Features::TIMESTAMP_QUERY
        | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES
        | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS
        | wgpu::Features::TEXTURE_BINDING_ARRAY
        | wgpu::Features::BUFFER_BINDING_ARRAY
        | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY
        | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
        | wgpu::Features::STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING
        | wgpu::Features::PIPELINE_STATISTICS_QUERY
}

pub fn negotiate_gpu_requirements(
    adapter_features: wgpu::Features,
    adapter_limits: &wgpu::Limits,
) -> Result<NegotiatedGpuRequirements, GpuRequirementsError> {
    let baseline = required_gpu_features();
    let missing = baseline & !adapter_features;
    if !missing.is_empty() {
        return Err(GpuRequirementsError::MissingRequiredFeatures(missing));
    }

    let mut optional_features = optional_gpu_features() & adapter_features;
    if !adapter_features.contains(wgpu::Features::TIMESTAMP_QUERY) {
        optional_features.remove(
            wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES
                | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
        );
    }
    if !adapter_features.contains(wgpu::Features::TEXTURE_BINDING_ARRAY) {
        optional_features.remove(wgpu::Features::STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING);
    }
    if !adapter_features.contains(wgpu::Features::BUFFER_BINDING_ARRAY) {
        optional_features.remove(
            wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY
                | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
        );
    }
    let enabled_features = baseline | optional_features;
    let required_limits = platform_limits_from_adapter_limits(platform_limits(), adapter_limits);
    if required_limits.max_storage_buffers_per_shader_stage
        < PORTABLE_MAX_STORAGE_BUFFERS_PER_SHADER_STAGE
    {
        return Err(GpuRequirementsError::InsufficientLimit {
            name: "max_storage_buffers_per_shader_stage",
            required: PORTABLE_MAX_STORAGE_BUFFERS_PER_SHADER_STAGE,
            available: adapter_limits.max_storage_buffers_per_shader_stage,
        });
    }
    if required_limits.max_bindings_per_bind_group < PORTABLE_MAX_BINDINGS_PER_SHADER_STAGE {
        return Err(GpuRequirementsError::InsufficientLimit {
            name: "max_bindings_per_bind_group",
            required: PORTABLE_MAX_BINDINGS_PER_SHADER_STAGE,
            available: adapter_limits.max_bindings_per_bind_group,
        });
    }

    Ok(NegotiatedGpuRequirements {
        required_features: enabled_features,
        required_limits,
        capabilities: RenderCapabilities::from_adapter(adapter_limits, enabled_features),
    })
}
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(target_os = "windows")]
pub use windows::*;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub use unix::*;
#[cfg(target_os = "linux")]
pub mod unix;

#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "android")]
pub use android::*;
#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "ios")]
pub use ios::*;
#[cfg(target_os = "ios")]
pub mod ios;

pub const WGPU_FIXED_ALIGNMENT: u64 = 16; // 4 bytes is min alignment for wgpu

pub fn platform_limits_from_adapter_limits(
    requested_limits: wgpu::Limits,
    adapter_limits: &wgpu::Limits,
) -> wgpu::Limits {
    // Start with adapter limits so every field is valid, then retain the
    // smaller application request for the fields used by the renderer.
    let mut limits = adapter_limits.clone();
    limits.max_storage_buffers_per_shader_stage = requested_limits
        .max_storage_buffers_per_shader_stage
        .min(adapter_limits.max_storage_buffers_per_shader_stage);
    limits.max_bindings_per_bind_group = requested_limits
        .max_bindings_per_bind_group
        .min(adapter_limits.max_bindings_per_bind_group);
    limits.max_storage_buffers_in_vertex_stage = requested_limits
        .max_storage_buffers_in_vertex_stage
        .min(adapter_limits.max_storage_buffers_in_vertex_stage);
    limits.max_storage_buffers_in_fragment_stage = requested_limits
        .max_storage_buffers_in_fragment_stage
        .min(adapter_limits.max_storage_buffers_in_fragment_stage);
    limits.max_binding_array_elements_per_shader_stage = requested_limits
        .max_binding_array_elements_per_shader_stage
        .min(adapter_limits.max_binding_array_elements_per_shader_stage);
    limits
}

pub fn shader_preprocessor_defs<const PLATFORM_TYPE: PlatformType>() -> Vec<String> {
    if PLATFORM_TYPE == PLATFORM_TYPE_PC {
        vec![
            "FEATURES_TEXTURE_BINDING_ARRAY".to_string(),
            "FEATURES_MULTISAMPLING".to_string(),
            "NO_FEATURES_PRIMITIVE_INDEX".to_string(),
            "FEATURES_MULTI_DRAW_INDIRECT_COUNT".to_string(),
        ]
    } else {
        vec![
            "NO_FEATURES_TEXTURE_BINDING_ARRAY".to_string(),
            "NO_FEATURES_MULTISAMPLING".to_string(),
            "NO_FEATURES_PRIMITIVE_INDEX".to_string(),
            "NO_FEATURES_MULTI_DRAW_INDIRECT_COUNT".to_string(),
        ]
    }
}
