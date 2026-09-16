pub const SIZE_OF_DATA_BUFFER_ELEMENT: usize = 4;
use inox_render::{AsBinding, RenderContext};
use inox_uid::{generate_static_uid_from_string, generate_uid_from_string, Uid};

pub const WAVEFRONT_QUEUE_VERSION: u32 = inox_render::WAVEFRONT_RECORD_VERSION;
pub const WAVEFRONT_INDIRECT_ACCUMULATOR_ID: Uid =
    generate_static_uid_from_string("WAVEFRONT_INDIRECT_ACCUMULATOR");
pub const WAVEFRONT_FINAL_ACCUMULATOR_ID: Uid =
    generate_static_uid_from_string("WAVEFRONT_FINAL_ACCUMULATOR");
pub const WAVEFRONT_UPSCALED_ACCUMULATOR_ID: Uid =
    generate_static_uid_from_string("WAVEFRONT_UPSCALED_ACCUMULATOR");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WavefrontQueueBindingIds {
    pub ray_queue: inox_uid::Uid,
    pub ray_header: inox_uid::Uid,
    pub hit_queue: inox_uid::Uid,
    pub hit_header: inox_uid::Uid,
    pub dispatch_args: inox_uid::Uid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WavefrontBounceBindingIds {
    pub source_ray_queue: inox_uid::Uid,
    pub source_hit_queue: inox_uid::Uid,
    pub source_hit_header: inox_uid::Uid,
    pub destination_ray_queue: inox_uid::Uid,
    pub destination_ray_header: inox_uid::Uid,
    pub dispatch_args: inox_uid::Uid,
}

pub fn wavefront_primary_bindings() -> WavefrontQueueBindingIds {
    WavefrontQueueBindingIds {
        ray_queue: crate::WAVEFRONT_PRIMARY_QUEUE_ID,
        ray_header: crate::WAVEFRONT_PRIMARY_HEADER_ID,
        hit_queue: crate::WAVEFRONT_HIT_QUEUE_ID,
        hit_header: crate::WAVEFRONT_HIT_HEADER_ID,
        dispatch_args: crate::WAVEFRONT_DISPATCH_ARGS_ID,
    }
}

pub fn wavefront_primary_shade_bindings() -> WavefrontQueueBindingIds {
    WavefrontQueueBindingIds {
        dispatch_args: crate::WAVEFRONT_HIT_DISPATCH_ARGS_ID,
        ..wavefront_primary_bindings()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WavefrontBounceStage {
    pub index: u32,
    pub name: String,
    pub dispatch_name: String,
    pub traversal_name: String,
    pub hit_dispatch_name: String,
    pub shade_name: String,
    pub bindings: WavefrontBounceBindingIds,
    pub traversal_bindings: WavefrontQueueBindingIds,
    pub shade_bindings: WavefrontQueueBindingIds,
    pub destination_dispatch_args: Uid,
}

/// Derive the queue, header, dispatch resource, and pass names for one
/// generated bounce stage. The viewer uses this metadata to construct the
/// same graph that the pass-order contract inspects.
pub fn wavefront_bounce_stage(
    index: u32,
    source_ray_queue: Uid,
    source_dispatch_args: Uid,
) -> WavefrontBounceStage {
    assert!(
        (1..=8).contains(&index),
        "wavefront bounce index must be 1..=8"
    );
    let zero_based = index - 1;
    let destination_ray_queue =
        generate_uid_from_string(&format!("WAVEFRONT_BOUNCE_{zero_based}_QUEUE"));
    let destination_ray_header =
        generate_uid_from_string(&format!("WAVEFRONT_BOUNCE_{zero_based}_HEADER"));
    let destination_dispatch_args =
        generate_uid_from_string(&format!("WAVEFRONT_BOUNCE_{zero_based}_DISPATCH"));
    WavefrontBounceStage {
        index,
        name: format!("ComputeWavefrontBounce{index}"),
        dispatch_name: format!("ComputeWavefrontBounce{index}Dispatch"),
        traversal_name: format!("ComputeWavefrontBounce{index}Traversal"),
        hit_dispatch_name: format!("ComputeWavefrontBounce{index}HitDispatch"),
        shade_name: format!("ComputeWavefrontBounce{index}Shade"),
        bindings: WavefrontBounceBindingIds {
            source_ray_queue,
            source_hit_queue: crate::WAVEFRONT_HIT_QUEUE_ID,
            source_hit_header: crate::WAVEFRONT_HIT_HEADER_ID,
            destination_ray_queue,
            destination_ray_header,
            dispatch_args: source_dispatch_args,
        },
        traversal_bindings: WavefrontQueueBindingIds {
            ray_queue: destination_ray_queue,
            ray_header: destination_ray_header,
            hit_queue: crate::WAVEFRONT_HIT_QUEUE_ID,
            hit_header: crate::WAVEFRONT_HIT_HEADER_ID,
            dispatch_args: destination_dispatch_args,
        },
        shade_bindings: WavefrontQueueBindingIds {
            ray_queue: destination_ray_queue,
            ray_header: destination_ray_header,
            hit_queue: crate::WAVEFRONT_HIT_QUEUE_ID,
            hit_header: crate::WAVEFRONT_HIT_HEADER_ID,
            dispatch_args: crate::WAVEFRONT_HIT_DISPATCH_ARGS_ID,
        },
        destination_dispatch_args,
    }
}

/// Build the complete generated bounce graph using the same stage inputs that
/// the viewer's bounce factory consumes.
pub fn wavefront_bounce_stages() -> Vec<WavefrontBounceStage> {
    let mut stages = Vec::with_capacity(8);
    let mut source_queue = crate::WAVEFRONT_PRIMARY_QUEUE_ID;
    let mut source_dispatch = crate::WAVEFRONT_HIT_DISPATCH_ARGS_ID;
    for index in 1..=8 {
        let stage = wavefront_bounce_stage(index, source_queue, source_dispatch);
        source_queue = stage.bindings.destination_ray_queue;
        source_dispatch = stage.destination_dispatch_args;
        stages.push(stage);
    }
    stages
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WavefrontAuxiliaryBindings {
    pub transparent_queue: Uid,
    pub transparent_header: Uid,
    pub ao_state: Uid,
}

/// Shared handoff IDs used by the transparency producer/resolve pair and the
/// AO producer/resolve pair.
pub fn wavefront_auxiliary_bindings() -> WavefrontAuxiliaryBindings {
    WavefrontAuxiliaryBindings {
        transparent_queue: crate::WAVEFRONT_TRANSPARENT_QUEUE_ID,
        transparent_header: crate::WAVEFRONT_TRANSPARENT_HEADER_ID,
        ao_state: crate::WAVEFRONT_AO_ID,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WavefrontPipelineDescriptor {
    pub pipeline: &'static str,
    pub shader: &'static str,
}

/// Pipeline descriptors referenced by active wavefront pass constructors.
/// Generated bounces reuse the next-bounce, dispatch, traversal, and shade
/// descriptors, so the descriptor table is the source of truth for shader
/// validation rather than a second hand-maintained shader inventory.
pub const WAVEFRONT_PRODUCTION_PIPELINES: &[WavefrontPipelineDescriptor] = &[
    WavefrontPipelineDescriptor {
        pipeline: crate::COMPUTE_WAVEFRONT_SURFACE_PIPELINE,
        shader: "compute_wavefront_surface.wgsl",
    },
    WavefrontPipelineDescriptor {
        pipeline: crate::COMPUTE_WAVEFRONT_TRANSPARENCY_PIPELINE,
        shader: "compute_wavefront_transparency.wgsl",
    },
    WavefrontPipelineDescriptor {
        pipeline: crate::COMPUTE_WAVEFRONT_AO_RESOLVE_PIPELINE,
        shader: "compute_wavefront_ao_resolve.wgsl",
    },
    WavefrontPipelineDescriptor {
        pipeline: crate::COMPUTE_WAVEFRONT_DIRECT_CANDIDATES_PIPELINE,
        shader: "compute_wavefront_direct_candidates.wgsl",
    },
    WavefrontPipelineDescriptor {
        pipeline: crate::COMPUTE_WAVEFRONT_VISIBILITY_PIPELINE,
        shader: "compute_wavefront_visibility.wgsl",
    },
    WavefrontPipelineDescriptor {
        pipeline: crate::COMPUTE_WAVEFRONT_PRIMARY_PIPELINE,
        shader: "compute_wavefront_primary.wgsl",
    },
    WavefrontPipelineDescriptor {
        pipeline: crate::COMPUTE_WAVEFRONT_DISPATCH_PIPELINE,
        shader: "compute_wavefront_dispatch.wgsl",
    },
    WavefrontPipelineDescriptor {
        pipeline: crate::COMPUTE_WAVEFRONT_TRAVERSAL_PIPELINE,
        shader: "compute_wavefront_traversal.wgsl",
    },
    WavefrontPipelineDescriptor {
        pipeline: crate::COMPUTE_WAVEFRONT_SHADE_PIPELINE,
        shader: "compute_wavefront_shade.wgsl",
    },
    WavefrontPipelineDescriptor {
        pipeline: crate::COMPUTE_WAVEFRONT_NEXT_BOUNCE_PIPELINE,
        shader: "compute_wavefront_next_bounce.wgsl",
    },
    WavefrontPipelineDescriptor {
        pipeline: crate::COMPUTE_WAVEFRONT_COMPOSITE_PIPELINE,
        shader: "compute_wavefront_composite.wgsl",
    },
    WavefrontPipelineDescriptor {
        pipeline: crate::COMPUTE_WAVEFRONT_TRANSPARENCY_RESOLVE_PIPELINE,
        shader: "compute_wavefront_transparency_resolve.wgsl",
    },
    WavefrontPipelineDescriptor {
        pipeline: crate::COMPUTE_WAVEFRONT_DENOISE_TEMPORAL_PIPELINE,
        shader: "compute_wavefront_denoise_temporal.wgsl",
    },
    WavefrontPipelineDescriptor {
        pipeline: crate::COMPUTE_WAVEFRONT_DENOISE_BILATERAL_PIPELINE,
        shader: "compute_wavefront_denoise_bilateral.wgsl",
    },
    WavefrontPipelineDescriptor {
        pipeline: crate::COMPUTE_WAVEFRONT_UPSCALE_PIPELINE,
        shader: "compute_wavefront_upscale.wgsl",
    },
];

pub fn wavefront_production_shader_files() -> Vec<&'static str> {
    let mut shaders = WAVEFRONT_PRODUCTION_PIPELINES
        .iter()
        .map(|descriptor| descriptor.shader)
        .collect::<Vec<_>>();
    shaders.sort_unstable();
    shaders.dedup();
    shaders
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct RayPackedData(pub f32);

/// A fixed-size ray record optimized for coalesced queue access.
#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct RayQueueRecord {
    pub origin_and_t_min: [f32; 4],
    pub direction_and_t_max: [f32; 4],
    pub throughput: [f32; 4],
    pub pixel_index: u32,
    pub bounce: u32,
    pub flags: u32,
    pub sample_index: u32,
}

/// A traversal result consumed by hit classification and surface preparation.
#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct HitRecord {
    pub distance: f32,
    pub visibility_id: u32,
    pub instance_id: u32,
    pub primitive_id: u32,
    pub barycentric: [f32; 2],
    pub ray_index: u32,
    pub flags: u32,
}

/// Surface-independent accumulation state used by temporal reconstruction.
#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct AccumulationRecord {
    pub radiance_and_weight: [f32; 4],
    pub first_and_second_moment: [f32; 2],
    pub history_length: u32,
    pub flags: u32,
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct ReservoirRecord {
    pub selected_light_id: u32,
    pub candidate_count: u32,
    pub weight_sum: f32,
    pub target_pdf: f32,
    pub selected_weight: f32,
    pub age: u32,
    pub flags: u32,
    pub confidence: f32,
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct AoRecord {
    pub visibility: f32,
    pub radius: f32,
    pub sample_count: u32,
    pub flags: u32,
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct TransparentHitRecord {
    pub pixel_index: u32,
    pub material_id: u32,
    pub depth: f32,
    pub alpha: f32,
    pub color: [f32; 4],
    pub layer: u32,
    pub flags: u32,
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct DispatchArgs {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

#[repr(C)]
#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct QueueHeader {
    pub version: u32,
    pub count: u32,
    pub capacity: u32,
    pub overflowed: u32,
}

impl QueueHeader {
    pub fn is_overflowed(self) -> bool {
        self.overflowed != 0
    }
}

/// A header read back from a submitted GPU frame. Keeping the complete header
/// makes both overflow and the amount of truncation observable to diagnostics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueueOverflowDiagnostic {
    pub name: String,
    pub header: QueueHeader,
}

/// Read every bound wavefront queue header from the GPU.
///
/// Header buffers retain `COPY_SRC`, and this function deliberately reads
/// the GPU buffer through a temporary staging allocation rather than the
/// CPU-side mirror. Call it after the
/// frame submission (the readback waits for completion on native backends).
pub fn read_wavefront_queue_diagnostics(
    render_context: &RenderContext,
) -> Vec<QueueOverflowDiagnostic> {
    let mut headers = vec![
        ("primary".to_string(), crate::WAVEFRONT_PRIMARY_HEADER_ID),
        ("hit".to_string(), crate::WAVEFRONT_HIT_HEADER_ID),
        (
            "transparency".to_string(),
            crate::WAVEFRONT_TRANSPARENT_HEADER_ID,
        ),
    ];
    headers.extend(wavefront_bounce_stages().into_iter().map(|stage| {
        (
            format!("bounce{}", stage.index),
            stage.bindings.destination_ray_header,
        )
    }));

    headers
        .into_iter()
        .filter_map(|(name, id)| {
            let header = render_context
                .global_buffers()
                .vector_with_id::<QueueHeader>(id)
                .read()
                .ok()?
                .buffer_id();
            let header = render_context
                .read_buffer_as::<QueueHeader>(header)?
                .into_iter()
                .next()?;
            Some(QueueOverflowDiagnostic { name, header })
        })
        .collect()
}

/// Read and publish every queue header after a submitted frame completes.
///
/// The staging readback waits for GPU completion, so this is safe to register
/// as a post-submit callback rather than inspecting the CPU-side mirror.
pub fn report_wavefront_queue_diagnostics(render_context: &RenderContext) {
    for diagnostic in read_wavefront_queue_diagnostics(render_context) {
        eprintln!(
            "wavefront queue {}: count={} capacity={} overflowed={}",
            diagnostic.name,
            diagnostic.header.count,
            diagnostic.header.capacity,
            diagnostic.header.is_overflowed(),
        );
    }
}

/// Finite per-queue budget supported by the portable renderer.  It covers
/// the default 1920x1080 frame (2,073,600 entries) while bounding GPU memory
/// and keeping header capacities representable as `u32`.
pub const MAX_WAVEFRONT_QUEUE_CAPACITY: usize = 2_097_152;

pub fn queue_capacity_for_resolution(
    dimensions: (u32, u32),
    render_scale: f32,
    max_capacity: usize,
) -> usize {
    let scale = render_scale.clamp(0.01, 1.0);
    let width = ((dimensions.0 as f32 * scale).ceil() as usize).max(1);
    let height = ((dimensions.1 as f32 * scale).ceil() as usize).max(1);
    width
        .saturating_mul(height)
        .min(max_capacity.max(1))
        .min(u32::MAX as usize)
}

/// Convert a queue length to the GPU header's bounded `u32` capacity field.
/// Values above the representable range are saturated; append shaders retain
/// their overflow marker for work beyond that capacity.
pub fn queue_capacity_u32(capacity: usize) -> u32 {
    u32::try_from(capacity).unwrap_or(u32::MAX)
}

pub fn dispatch_workgroups_for(item_count: u32, workgroup_size: u32) -> u32 {
    if item_count == 0 || workgroup_size == 0 {
        0
    } else {
        item_count.div_ceil(workgroup_size)
    }
}

/// Clamp transport controls to the bounded queue graph supported by the renderer.
pub fn bounded_path_depth(max_depth: u32) -> u32 {
    max_depth.min(8)
}

/// Return the unbiased survival probability used by the shader roulette step.
pub fn roulette_survival_probability(throughput: [f32; 3]) -> f32 {
    let maximum = throughput
        .into_iter()
        .map(|value| {
            if value.is_finite() {
                value.max(0.0)
            } else {
                0.0
            }
        })
        .fold(0.0, f32::max);
    maximum.clamp(0.05, 0.99)
}

pub fn reflect_direction(incoming: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    let dot = incoming[0] * normal[0] + incoming[1] * normal[1] + incoming[2] * normal[2];
    [
        incoming[0] - 2.0 * dot * normal[0],
        incoming[1] - 2.0 * dot * normal[1],
        incoming[2] - 2.0 * dot * normal[2],
    ]
}

/// Return eta_i / eta_t for a ray crossing an air/material boundary.
///
/// `front_face` is true when the ray enters the material. The shader uses the
/// same convention after orienting the shading normal against the incident
/// direction.
pub fn ior_ratio(front_face: bool, material_ior: f32) -> f32 {
    let ior = if material_ior.is_finite() {
        material_ior.clamp(1.001, 3.0)
    } else {
        1.5
    };
    if front_face {
        1.0 / ior
    } else {
        ior
    }
}

/// Compute refraction, returning None for total internal reflection.
pub fn refract_direction(incoming: [f32; 3], normal: [f32; 3], eta: f32) -> Option<[f32; 3]> {
    let eta = if eta.is_finite() {
        eta.max(f32::EPSILON)
    } else {
        1.0
    };
    let cosine = -(incoming[0] * normal[0] + incoming[1] * normal[1] + incoming[2] * normal[2]);
    let k = 1.0 - eta * eta * (1.0 - cosine * cosine);
    if k <= 0.0 {
        return None;
    }
    Some([
        eta * incoming[0] + (eta * cosine - k.sqrt()) * normal[0],
        eta * incoming[1] + (eta * cosine - k.sqrt()) * normal[1],
        eta * incoming[2] + (eta * cosine - k.sqrt()) * normal[2],
    ])
}

/// CPU-side mirror of the GPU append queue used for sizing and deterministic
/// tests. The GPU implementation uses the same header semantics.
pub struct RayQueue<T> {
    records: Vec<T>,
    header: QueueHeader,
}

impl<T> RayQueue<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            records: Vec::with_capacity(capacity),
            header: QueueHeader {
                version: WAVEFRONT_QUEUE_VERSION,
                count: 0,
                capacity: capacity.min(u32::MAX as usize) as u32,
                overflowed: 0,
            },
        }
    }

    pub fn append(&mut self, record: T) -> bool {
        if self.records.len() >= self.records.capacity() {
            self.header.overflowed = 1;
            return false;
        }
        self.records.push(record);
        self.header.count = self.records.len() as u32;
        true
    }

    pub fn reset(&mut self) {
        self.records.clear();
        self.header.count = 0;
        self.header.overflowed = 0;
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn is_overflowed(&self) -> bool {
        self.header.overflowed != 0
    }

    pub fn header(&self) -> QueueHeader {
        self.header
    }

    pub fn records(&self) -> &[T] {
        &self.records
    }
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct RadiancePackedData(pub f32);

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct ThroughputPackedData(pub f32);

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct DebugPackedData(pub f32);
