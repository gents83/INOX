pub const SIZE_OF_DATA_BUFFER_ELEMENT: usize = 4;
#[cfg(not(target_arch = "wasm32"))]
use crate::{WAVEFRONT_AO_ID, WAVEFRONT_DIRECT_ACCUMULATOR_ID, WAVEFRONT_SCENE_ID};
#[cfg(not(target_arch = "wasm32"))]
use inox_render::AsBinding;
use inox_render::RenderContext;
use inox_uid::{generate_static_uid_from_string, generate_uid_from_string, Uid};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex, OnceLock,
};

pub const WAVEFRONT_QUEUE_VERSION: u32 = inox_render::WAVEFRONT_RECORD_VERSION;
pub const WAVEFRONT_INDIRECT_ACCUMULATOR_ID: Uid =
    generate_static_uid_from_string("WF_INDIRECT_ACCUMULATOR");
pub const WAVEFRONT_FINAL_ACCUMULATOR_ID: Uid =
    generate_static_uid_from_string("WF_FINAL_ACCUMULATOR");
pub const WAVEFRONT_DENOISE_HISTORY_ID: Uid = generate_static_uid_from_string("WF_DENOISE_HISTORY");
pub const WAVEFRONT_UPSCALED_ACCUMULATOR_ID: Uid =
    generate_static_uid_from_string("WF_UPSCALED_ACCUMULATOR");

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

/// The generated bounce graph needs one source queue for the primary stage and
/// two destination queues for sequential ping-pong. Headers and dispatch args
/// remain stage-specific because they are written/read by different generated
/// passes, but the ray payload storage is never live for two adjacent stages at
/// once.
pub const WAVEFRONT_BOUNCE_QUEUE_COUNT: usize = 2;

pub fn wavefront_bounce_queue_id(index: u32) -> Uid {
    assert!(
        (1..=8).contains(&index),
        "wavefront bounce index must be 1..=8"
    );
    match (index - 1) as usize % WAVEFRONT_BOUNCE_QUEUE_COUNT {
        0 => crate::WAVEFRONT_NEXT_QUEUE_ID,
        1 => crate::WAVEFRONT_NEXT2_QUEUE_ID,
        _ => unreachable!("ping-pong queue index is bounded to two queues"),
    }
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
    let destination_ray_queue = wavefront_bounce_queue_id(index);
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
    for index in 1..=8 {
        // Every bounce consumes the shared hit-dispatch arguments written by
        // its preceding hit-dispatch pass.  The per-bounce destination args
        // are only the traversal producer for the next stage.
        let stage =
            wavefront_bounce_stage(index, source_queue, crate::WAVEFRONT_HIT_DISPATCH_ARGS_ID);
        source_queue = stage.bindings.destination_ray_queue;
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
        pipeline: crate::DEBUG_PIPELINE,
        shader: "debug.wgsl",
    },
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
    pub requested_capacity: u32,
    pub dropped_records: u32,
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
    pub requested_capacity: u32,
    pub actual_capacity: u32,
    pub overflowed: bool,
    pub dropped_records: u32,
    pub active_bounce_count: u32,
    pub header: QueueHeader,
}

/// The fixed graph has three auxiliary headers and one destination header per
/// generated bounce. A successful diagnostic must contain every one of them.
#[cfg(not(target_arch = "wasm32"))]
pub const EXPECTED_WAVEFRONT_QUEUE_HEADER_COUNT: usize = 3 + 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QueueDiagnosticMarkerState {
    pub complete: bool,
    pub no_overflow: bool,
}

pub fn queue_diagnostic_marker_state(
    diagnostics: &[QueueOverflowDiagnostic],
    expected: usize,
) -> QueueDiagnosticMarkerState {
    let complete = diagnostics.len() == expected;
    QueueDiagnosticMarkerState {
        complete,
        no_overflow: complete
            && diagnostics.iter().all(|diagnostic| {
                !diagnostic.header.is_overflowed() && diagnostic.header.dropped_records == 0
            }),
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueueHeaderReadbackError {
    pub expected: usize,
    pub received: usize,
    pub missing: Vec<String>,
}

/// Read every bound wavefront queue header from the GPU.
///
/// Header buffers retain `COPY_SRC`, and this function deliberately reads
/// the GPU buffer through a temporary staging allocation rather than the
/// CPU-side mirror. Call it after the frame submission (the readback waits
/// for completion on native backends). WebGPU/WASM does not expose the
/// synchronous poll needed by this path, so diagnostics are native-only.
#[cfg(not(target_arch = "wasm32"))]
pub fn read_wavefront_queue_diagnostics(
    render_context: &RenderContext,
) -> Result<Vec<QueueOverflowDiagnostic>, QueueHeaderReadbackError> {
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

    let active_bounce_count = render_context
        .global_buffers()
        .constant_data
        .read()
        .map(|data| data.num_bounces())
        .unwrap_or(0);
    let expected = EXPECTED_WAVEFRONT_QUEUE_HEADER_COUNT;
    if headers.len() != expected {
        return Err(QueueHeaderReadbackError {
            expected,
            received: headers.len(),
            missing: vec!["wavefront-graph-header-set".to_string()],
        });
    }
    let mut missing = Vec::new();
    let mut diagnostics = Vec::with_capacity(expected);
    for (name, id) in headers {
        let Some(header_buffer) = render_context
            .global_buffers()
            .vector_with_id::<QueueHeader>(id)
            .read()
            .ok()
            .map(|buffer| buffer.buffer_id())
        else {
            missing.push(name);
            continue;
        };
        let Some(header) = render_context
            .read_buffer_as::<QueueHeader>(header_buffer)
            .and_then(|headers| headers.into_iter().next())
        else {
            missing.push(name);
            continue;
        };
        diagnostics.push(QueueOverflowDiagnostic {
            name,
            requested_capacity: header.requested_capacity,
            actual_capacity: header.capacity,
            overflowed: header.is_overflowed(),
            dropped_records: header.dropped_records,
            active_bounce_count,
            header,
        });
    }
    if diagnostics.len() != expected {
        return Err(QueueHeaderReadbackError {
            expected,
            received: diagnostics.len(),
            missing,
        });
    }
    Ok(diagnostics)
}

/// Result of the opt-in native Cornell-box image diagnostic.
#[derive(Clone, Debug, PartialEq)]
pub struct CornellImageDiagnostic {
    pub finite: bool,
    pub coverage: f32,
    pub mean_luminance: f32,
    pub non_black: bool,
    pub non_white: bool,
    pub red_wall_band: usize,
    pub green_wall_band: usize,
    pub red_green_separation: bool,
    pub bright_ceiling: bool,
}

impl CornellImageDiagnostic {
    pub fn passed(&self) -> bool {
        self.finite
            && self.coverage > 0.01
            && self.non_black
            && self.non_white
            && self.red_green_separation
            && self.bright_ceiling
    }

    pub fn summary(&self) -> String {
        format!(
            "finite={} coverage={:.3} mean_luminance={:.3} non_black={} non_white={} red_band={} green_band={} red_green_separation={} bright_ceiling={}",
            self.finite,
            self.coverage,
            self.mean_luminance,
            self.non_black,
            self.non_white,
            self.red_wall_band,
            self.green_wall_band,
            self.red_green_separation,
            self.bright_ceiling,
        )
    }
}

/// Check the spatial/color invariants of a Cornell-like linear HDR image.
///
/// Inputs are RGBA float pixels in row-major top-to-bottom order. Values are tone-mapped only
/// for metrics, so the thresholds remain stable for HDR accumulators. The fixture assumes the
/// camera sees both side walls and the ceiling light, but permits horizontal mirroring by looking
/// for the strongest red and green response in separate thirds rather than fixed left/right sides.
pub fn diagnose_cornell_image(width: u32, height: u32, pixels: &[f32]) -> CornellImageDiagnostic {
    let pixel_count = width as usize * height as usize;
    let finite = pixels.len() == pixel_count * 4 && pixels.iter().all(|value| value.is_finite());
    if width == 0 || height == 0 || pixels.len() < pixel_count * 4 {
        return CornellImageDiagnostic {
            finite,
            coverage: 0.0,
            mean_luminance: 0.0,
            non_black: false,
            non_white: true,
            red_wall_band: usize::MAX,
            green_wall_band: usize::MAX,
            red_green_separation: false,
            bright_ceiling: false,
        };
    }

    let mut sum_luminance = 0.0;
    let mut covered = 0usize;
    let mut bands = [[0.0f32; 3]; 3];
    let mut band_counts = [0usize; 3];
    let mut top_sum = 0.0;
    let mut lower_sum = 0.0;
    for y in 0..height as usize {
        for x in 0..width as usize {
            let offset = (y * width as usize + x) * 4;
            let rgb = [pixels[offset], pixels[offset + 1], pixels[offset + 2]];
            let mapped = rgb.map(|value| value.max(0.0) / (1.0 + value.max(0.0)));
            let luminance = 0.2126 * mapped[0] + 0.7152 * mapped[1] + 0.0722 * mapped[2];
            sum_luminance += luminance;
            covered += usize::from(luminance > 0.002);
            let band = (x * 3 / width as usize).min(2);
            bands[band]
                .iter_mut()
                .zip(mapped)
                .for_each(|(dst, value)| *dst += value);
            band_counts[band] += 1;
            if y * 4 < height as usize {
                top_sum += luminance;
            } else {
                lower_sum += luminance;
            }
        }
    }
    for (band, count) in bands.iter_mut().zip(band_counts) {
        if count != 0 {
            band.iter_mut().for_each(|value| *value /= count as f32);
        }
    }
    let mean_luminance = sum_luminance / pixel_count as f32;
    let coverage = covered as f32 / pixel_count as f32;
    let red_scores = bands.map(|band| band[0] - band[1].max(band[2]));
    let green_scores = bands.map(|band| band[1] - band[0].max(band[2]));
    let red_wall_band = red_scores
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
        .unwrap_or(usize::MAX);
    let green_wall_band = green_scores
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
        .unwrap_or(usize::MAX);
    let red_green_separation = red_wall_band.abs_diff(green_wall_band) >= 1
        && red_scores[red_wall_band] > 0.02
        && green_scores[green_wall_band] > 0.02;
    let top_mean = top_sum / (pixel_count as f32 * 0.25);
    let lower_mean = lower_sum / (pixel_count as f32 * 0.75);

    CornellImageDiagnostic {
        finite,
        coverage,
        mean_luminance,
        non_black: coverage > 0.01 && mean_luminance > 0.005,
        non_white: mean_luminance < 0.95,
        red_wall_band,
        green_wall_band,
        red_green_separation,
        bright_ceiling: top_mean > 0.12 && top_mean > lower_mean * 1.10 + 0.01,
    }
}

/// Read and publish every queue header after a submitted frame completes.
///
/// The staging readback waits for GPU completion, so this is safe to register
/// as a post-submit callback rather than inspecting the CPU-side mirror.
#[cfg(not(target_arch = "wasm32"))]
pub fn report_wavefront_queue_diagnostics(render_context: &RenderContext) {
    static OVERFLOW_STATES: OnceLock<Mutex<HashMap<String, bool>>> = OnceLock::new();
    static HEADER_COMPLETE_MARKER_EMITTED: OnceLock<()> = OnceLock::new();
    static NO_OVERFLOW_MARKER_EMITTED: OnceLock<()> = OnceLock::new();
    let states = OVERFLOW_STATES.get_or_init(|| Mutex::new(HashMap::new()));
    let diagnostics = match read_wavefront_queue_diagnostics(render_context) {
        Ok(diagnostics) => diagnostics,
        Err(error) => {
            eprintln!(
                "INOX_NATIVE_WAVEFRONT_QUEUE_HEADERS_FAILED expected={} received={} missing={}",
                error.expected,
                error.received,
                error.missing.join(","),
            );
            return;
        }
    };
    let marker_state =
        queue_diagnostic_marker_state(&diagnostics, EXPECTED_WAVEFRONT_QUEUE_HEADER_COUNT);
    if marker_state.complete && HEADER_COMPLETE_MARKER_EMITTED.set(()).is_ok() {
        eprintln!(
            "INOX_NATIVE_WAVEFRONT_QUEUE_HEADERS_COMPLETE count={} expected={} active_bounces={}",
            diagnostics.len(),
            EXPECTED_WAVEFRONT_QUEUE_HEADER_COUNT,
            diagnostics
                .first()
                .map(|diagnostic| diagnostic.active_bounce_count)
                .unwrap_or(0),
        );
    }
    if marker_state.no_overflow && NO_OVERFLOW_MARKER_EMITTED.set(()).is_ok() {
        eprintln!(
            "INOX_NATIVE_WAVEFRONT_QUEUE_NO_OVERFLOW count={} expected={} active_bounces={}",
            diagnostics.len(),
            EXPECTED_WAVEFRONT_QUEUE_HEADER_COUNT,
            diagnostics
                .first()
                .map(|diagnostic| diagnostic.active_bounce_count)
                .unwrap_or(0),
        );
    }
    let Ok(mut states) = states.lock() else {
        return;
    };

    for diagnostic in diagnostics {
        let overflowed = diagnostic.header.is_overflowed();
        let was_overflowed = states
            .insert(diagnostic.name.clone(), overflowed)
            .unwrap_or(false);
        // Report only transitions into overflow. Persistent overflow is deduplicated
        // until the queue recovers, keeping frame-by-frame diagnostics usable.
        if overflowed && !was_overflowed {
            eprintln!(
                "wavefront queue {} overflowed: count={} capacity={} requested={} dropped={} active_bounces={}",
                diagnostic.name,
                diagnostic.header.count,
                diagnostic.actual_capacity,
                diagnostic.requested_capacity,
                diagnostic.dropped_records,
                diagnostic.active_bounce_count,
            );
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn report_cornell_algorithm_diagnostics(render_context: &RenderContext) {
    use std::collections::BTreeMap;

    let buffers = render_context.global_buffers();
    let world = buffers.vector::<RayPackedData>();
    let material = buffers.vector::<RadiancePackedData>();
    let world_id = world.read().ok().map(|data| data.buffer_id());
    let material_id = material.read().ok().map(|data| data.buffer_id());
    let Some(world) = world_id.and_then(|id| render_context.read_buffer_as::<f32>(id)) else {
        eprintln!("INOX_NATIVE_CORNELL_ALGORITHM_DIAGNOSTICS gbuffer=unavailable");
        return;
    };
    let Some(material) = material_id.and_then(|id| render_context.read_buffer_as::<f32>(id)) else {
        eprintln!("INOX_NATIVE_CORNELL_ALGORITHM_DIAGNOSTICS material_buffer=unavailable");
        return;
    };
    let world_nonzero = world
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|record| record[..3].iter().any(|value| *value != 0.0))
        .count();
    eprintln!(
        "INOX_NATIVE_CORNELL_GBUFFER_WORLD_DIAGNOSTICS records={} nonzero={}",
        world.len() / 4,
        world_nonzero
    );
    let mut histogram = BTreeMap::<u32, usize>::new();
    let mut material_pixels = 0usize;
    for value in material.as_chunks::<4>().0.iter().map(|record| record[0]) {
        if value.is_finite() && (0.0..1024.0).contains(&value) && value.fract() == 0.0 {
            *histogram.entry(value as u32).or_default() += 1;
            material_pixels += 1;
        }
    }
    let scene = buffers.vector_with_id::<u32>(WAVEFRONT_SCENE_ID);
    let scene_words = scene.read().map(|data| data.clone()).unwrap_or_default();
    let table = |kind: usize| -> Option<(usize, usize, usize)> {
        let base = 4usize.checked_add(kind.checked_mul(4)?)?;
        let offset = *scene_words.get(base)? as usize;
        let count = *scene_words.get(base + 1)? as usize;
        let stride = *scene_words.get(base + 2)? as usize;
        (offset.checked_add(count)? <= scene_words.len()).then_some((offset, count, stride))
    };
    let mut instance_meshes = BTreeMap::<u32, usize>::new();
    if let Some((offset, count, stride)) = table(3) {
        for base in (offset..offset + count).step_by(stride.max(1)) {
            if let Some(value) = scene_words.get(base + 1).copied() {
                *instance_meshes.entry(value).or_default() += 1;
            }
        }
    }
    eprintln!(
        "INOX_NATIVE_CORNELL_INSTANCE_MESH_DIAGNOSTICS histogram={:?}",
        instance_meshes
    );
    let mut mesh_materials = BTreeMap::<i32, usize>::new();
    if let Some((offset, count, stride)) = table(5) {
        for base in (offset..offset + count).step_by(stride.max(1)) {
            if let Some(value) = scene_words.get(base + 3).copied() {
                *mesh_materials.entry(value as i32).or_default() += 1;
            }
        }
    }
    eprintln!(
        "INOX_NATIVE_CORNELL_MESH_MATERIAL_DIAGNOSTICS histogram={:?}",
        mesh_materials
    );
    let mut emissive_ids = Vec::new();
    if let Some((offset, count, stride)) = table(8) {
        for id in 0..(count / stride.max(1)) {
            let base = offset + id * stride;
            let strength = scene_words
                .get(base + 27)
                .copied()
                .map(f32::from_bits)
                .unwrap_or(0.0);
            let color = [24usize, 25, 26].map(|index| {
                scene_words
                    .get(base + index)
                    .copied()
                    .map(f32::from_bits)
                    .unwrap_or(0.0)
            });
            if strength.is_finite() && strength > 0.0 && color.iter().any(|value| *value > 0.0) {
                emissive_ids.push(id as u32);
            }
        }
    }
    let emissive_pixels = material
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|record| record[0].is_finite() && emissive_ids.contains(&(record[0] as u32)))
        .count();

    let describe_accumulator = |name: &str, id| {
        let Some(records) = buffers
            .vector_with_id::<AccumulationRecord>(id)
            .read()
            .ok()
            .map(|data| data.buffer_id())
            .and_then(|buffer| render_context.read_buffer_as::<AccumulationRecord>(buffer))
        else {
            eprintln!("{name}=unavailable");
            return;
        };
        let mut nonzero = 0usize;
        let mut sum = [0.0f64; 3];
        let mut max = [0.0f32; 3];
        for record in &records {
            if record.radiance_and_weight[..3]
                .iter()
                .any(|value| *value != 0.0)
            {
                nonzero += 1;
            }
            for channel in 0..3 {
                sum[channel] += record.radiance_and_weight[channel] as f64;
                max[channel] = max[channel].max(record.radiance_and_weight[channel]);
            }
        }
        let count = records.len().max(1) as f64;
        eprintln!(
            "{name}=pixels:{} nonzero:{} mean=({:.4},{:.4},{:.4}) max=({:.4},{:.4},{:.4})",
            records.len(),
            nonzero,
            sum[0] / count,
            sum[1] / count,
            sum[2] / count,
            max[0],
            max[1],
            max[2]
        );
    };

    let mut lights = String::new();
    if let Some((offset, count, stride)) = table(10) {
        let light_count = count / stride.max(1);
        for light_id in 0..light_count.min(4) {
            let base = offset + light_id * stride;
            let field = |index: usize| {
                scene_words
                    .get(base + index)
                    .copied()
                    .map(f32::from_bits)
                    .unwrap_or(0.0)
            };
            lights.push_str(&format!(
                " id{}=p({:.3},{:.3},{:.3}) intensity={:.3} c({:.3},{:.3},{:.3});",
                light_id,
                field(0),
                field(1),
                field(2),
                field(7),
                field(8),
                field(9),
                field(10)
            ));
        }
        eprintln!(
            "INOX_NATIVE_CORNELL_REPRESENTATIVE_LIGHTS count={}{}",
            light_count, lights
        );
    } else {
        eprintln!("INOX_NATIVE_CORNELL_REPRESENTATIVE_LIGHTS unavailable");
    }
    eprintln!(
        "INOX_NATIVE_CORNELL_MATERIAL_DIAGNOSTICS pixels={} histogram={:?} emissive_ids={:?} emissive_pixels={}",
        material_pixels, histogram, emissive_ids, emissive_pixels
    );
    let ao = buffers.vector_with_id::<AoRecord>(WAVEFRONT_AO_ID);
    if let Some(buffer) = ao.read().ok().map(|data| data.buffer_id()) {
        if let Some(records) = render_context.read_buffer_as::<AoRecord>(buffer) {
            let finite = records
                .iter()
                .filter(|record| record.visibility.is_finite())
                .count();
            let mean = records.iter().map(|record| record.visibility).sum::<f32>()
                / records.len().max(1) as f32;
            eprintln!(
                "INOX_NATIVE_CORNELL_VISIBILITY_DIAGNOSTICS ao_pixels={} finite={} mean_ao={:.4}",
                records.len(),
                finite,
                mean
            );
        }
    }
    describe_accumulator(
        "INOX_NATIVE_CORNELL_DIRECT_DIAGNOSTICS",
        WAVEFRONT_DIRECT_ACCUMULATOR_ID,
    );
    describe_accumulator(
        "INOX_NATIVE_CORNELL_INDIRECT_DIAGNOSTICS",
        WAVEFRONT_INDIRECT_ACCUMULATOR_ID,
    );
    eprintln!(
        "INOX_NATIVE_CORNELL_GBUFFER_DIAGNOSTICS world_words={} material_words={}",
        world.len(),
        material.len()
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn save_cornell_readback_ppm(target: usize, width: u32, height: u32, pixels: &[f32], first: bool) {
    let suffix = if first { "first" } else { "latest" };
    let path = std::env::temp_dir().join(format!("inox-cornell-target-{target}-{suffix}.ppm"));
    let mut ppm = format!("P6\n{width} {height}\n255\n").into_bytes();
    ppm.reserve(
        (width as usize)
            .saturating_mul(height as usize)
            .saturating_mul(3),
    );
    for pixel in pixels.as_chunks::<4>().0 {
        let tone_map = |value: f32| {
            let value = if value.is_finite() && value >= 0.0 {
                value
            } else {
                0.0
            };
            (value / (1.0 + value)).powf(1.0 / 2.2).clamp(0.0, 1.0)
        };
        ppm.extend([
            (tone_map(pixel[0]) * 255.0 + 0.5) as u8,
            (tone_map(pixel[1]) * 255.0 + 0.5) as u8,
            (tone_map(pixel[2]) * 255.0 + 0.5) as u8,
        ]);
    }
    if std::fs::write(&path, ppm).is_ok() {
        eprintln!(
            "INOX_NATIVE_CORNELL_IMAGE_PPM target={} kind={} path={}",
            target,
            suffix,
            path.display()
        );
    }
}

/// Run the bounded native image smoke against either ping-pong display target.
#[cfg(not(target_arch = "wasm32"))]
pub fn report_cornell_image_diagnostics(render_context: &RenderContext) -> bool {
    static FIRST_PPM_SAVED: AtomicBool = AtomicBool::new(false);
    static ALGORITHM_DIAGNOSTICS_REPORTED: AtomicBool = AtomicBool::new(false);
    let first = !FIRST_PPM_SAVED.swap(true, Ordering::AcqRel);
    if !ALGORITHM_DIAGNOSTICS_REPORTED.swap(true, Ordering::AcqRel) {
        report_cornell_algorithm_diagnostics(render_context);
    }
    let mut last = None;
    for target in [3usize, 4usize] {
        let Some((width, height, pixels)) = render_context.read_render_target_rgba32f(target)
        else {
            continue;
        };
        save_cornell_readback_ppm(target, width, height, &pixels, first);
        let diagnostic = diagnose_cornell_image(width, height, &pixels);
        if diagnostic.passed() {
            eprintln!(
                "INOX_NATIVE_CORNELL_IMAGE_PASS target={} {}",
                target,
                diagnostic.summary()
            );
            return true;
        }
        last = Some((target, diagnostic));
    }
    if let Some((target, diagnostic)) = last {
        eprintln!(
            "INOX_NATIVE_CORNELL_IMAGE_FAILED target={} {}",
            target,
            diagnostic.summary()
        );
    } else {
        eprintln!("INOX_NATIVE_CORNELL_IMAGE_READBACK_FAILED");
    }
    false
}

/// Queue readback is unavailable on pinned WebGPU/WASM; keep this path silent.
#[cfg(target_arch = "wasm32")]
pub fn report_wavefront_queue_diagnostics(_render_context: &RenderContext) {
    // WebGPU/WASM has no synchronous poll/readback path. Do not publish a
    // queue-header success marker until asynchronous GPU validation exists.
}

/// Aggregate queue budget for portable/constrained adapters with roughly 512
/// MiB available. This includes primary plus two ping-pong ray queues, the
/// shared hit queue, and the four-layer transparency queue. `GPUVector` keeps
/// a CPU `Vec` mirror for every GPU queue, so the aggregate estimate includes
/// both copies.
pub const WAVEFRONT_QUEUE_MEMORY_BUDGET_BYTES: usize = 512 * 1024 * 1024;
/// Discrete native adapters get a bounded 2 GiB aggregate fallback budget,
/// enough to cover the tested 1916x1125 maximize extent. Explicit negotiated
/// adapter byte limits still win, so constrained native adapters remain
/// bounded; this is not a physical-VRAM guarantee.
pub const WAVEFRONT_NATIVE_QUEUE_MEMORY_BUDGET_BYTES: usize = 2 * 1024 * 1024 * 1024;
pub const WAVEFRONT_QUEUE_GPU_BYTES_PER_CAPACITY_ENTRY: usize = (WAVEFRONT_BOUNCE_QUEUE_COUNT + 1)
    * std::mem::size_of::<RayQueueRecord>()
    + std::mem::size_of::<HitRecord>()
    + 4 * std::mem::size_of::<TransparentHitRecord>();
pub const WAVEFRONT_QUEUE_HOST_BYTES_PER_CAPACITY_ENTRY: usize =
    WAVEFRONT_QUEUE_GPU_BYTES_PER_CAPACITY_ENTRY;
pub const WAVEFRONT_QUEUE_BYTES_PER_CAPACITY_ENTRY: usize =
    WAVEFRONT_QUEUE_GPU_BYTES_PER_CAPACITY_ENTRY + WAVEFRONT_QUEUE_HOST_BYTES_PER_CAPACITY_ENTRY;
pub const WAVEFRONT_QUEUE_FALLBACK_CAPACITY: usize =
    WAVEFRONT_QUEUE_MEMORY_BUDGET_BYTES / WAVEFRONT_QUEUE_BYTES_PER_CAPACITY_ENTRY;
#[cfg(not(target_arch = "wasm32"))]
pub const WAVEFRONT_NATIVE_QUEUE_FALLBACK_CAPACITY: usize =
    WAVEFRONT_NATIVE_QUEUE_MEMORY_BUDGET_BYTES / WAVEFRONT_QUEUE_BYTES_PER_CAPACITY_ENTRY;
/// Hard ceiling used by error recovery. Keep it at least as high as the
/// discrete-native fallback so a calculation failure cannot silently lower
/// that negotiated policy; portable/WASM callers remain capped by their
/// 512 MiB fallback limits.
pub const MAX_WAVEFRONT_QUEUE_CAPACITY: usize =
    WAVEFRONT_NATIVE_QUEUE_MEMORY_BUDGET_BYTES / WAVEFRONT_QUEUE_BYTES_PER_CAPACITY_ENTRY;

pub fn wavefront_queue_memory_bytes(capacity: usize) -> usize {
    capacity.saturating_mul(WAVEFRONT_QUEUE_BYTES_PER_CAPACITY_ENTRY)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapacityError {
    ArithmeticOverflow,
    ZeroRecordSize,
    ZeroFallbackCapacity,
}

impl std::fmt::Display for CapacityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArithmeticOverflow => write!(f, "queue capacity arithmetic overflowed"),
            Self::ZeroRecordSize => write!(f, "queue record size must be non-zero"),
            Self::ZeroFallbackCapacity => write!(f, "queue fallback capacity must be non-zero"),
        }
    }
}

impl std::error::Error for CapacityError {}

/// Calculate a queue's requested and allocatable entry count using checked
/// arithmetic. Adapter byte limits are negotiated at device creation; a zero
/// limit means unavailable and uses the finite portable fallback.
pub fn queue_capacity_for_resolution(
    width: u32,
    height: u32,
    record_size: u32,
    multiplier: u32,
    limits: inox_render::QueueCapacityLimits,
) -> Result<u32, CapacityError> {
    if limits.fallback_capacity == 0 {
        return Err(CapacityError::ZeroFallbackCapacity);
    }
    if width == 0 || height == 0 || multiplier == 0 {
        return Ok(0);
    }
    if record_size == 0 {
        return Err(CapacityError::ZeroRecordSize);
    }
    let requested = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| pixels.checked_mul(u64::from(multiplier)))
        .ok_or(CapacityError::ArithmeticOverflow)?;
    let fallback = u64::from(limits.fallback_capacity);
    let storage_entries = if limits.max_storage_buffer_binding_size == 0 {
        fallback
    } else {
        limits.max_storage_buffer_binding_size / u64::from(record_size)
    };
    let buffer_entries = if limits.max_buffer_size == 0 {
        fallback
    } else {
        limits.max_buffer_size / u64::from(record_size)
    };
    let actual = requested
        .min(fallback)
        .min(storage_entries)
        .min(buffer_entries)
        .min(u64::from(u32::MAX));
    Ok(actual as u32)
}

pub fn queue_requested_capacity(
    width: u32,
    height: u32,
    multiplier: u32,
) -> Result<u32, CapacityError> {
    u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| pixels.checked_mul(u64::from(multiplier)))
        .ok_or(CapacityError::ArithmeticOverflow)
        .map(|value| value.min(u64::from(u32::MAX)) as u32)
}

pub fn portable_queue_capacity_limits() -> inox_render::QueueCapacityLimits {
    inox_render::QueueCapacityLimits {
        fallback_capacity: WAVEFRONT_QUEUE_FALLBACK_CAPACITY as u32,
        ..Default::default()
    }
}

pub fn queue_fallback_capacity_for_adapter(discrete_adapter: bool) -> usize {
    #[cfg(not(target_arch = "wasm32"))]
    if discrete_adapter {
        return WAVEFRONT_NATIVE_QUEUE_FALLBACK_CAPACITY;
    }
    let _ = discrete_adapter;
    WAVEFRONT_QUEUE_FALLBACK_CAPACITY
}

pub fn negotiated_queue_capacity_limits(
    render_context: &RenderContext,
) -> inox_render::QueueCapacityLimits {
    let mut limits = inox_render::QueueCapacityLimits::from(render_context.capabilities());
    limits.fallback_capacity = queue_capacity_u32(queue_fallback_capacity_for_adapter(
        render_context.capabilities().discrete_adapter,
    ));
    limits
}

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
                requested_capacity: capacity.min(u32::MAX as usize) as u32,
                dropped_records: 0,
            },
        }
    }

    pub fn append(&mut self, record: T) -> bool {
        if self.records.len() >= self.records.capacity() {
            self.header.overflowed = 1;
            self.header.dropped_records = self.header.dropped_records.saturating_add(1);
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
        self.header.dropped_records = 0;
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
