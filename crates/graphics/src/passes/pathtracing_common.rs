pub const SIZE_OF_DATA_BUFFER_ELEMENT: usize = 4;
use inox_uid::{generate_static_uid_from_string, generate_uid_from_string, Uid};

pub const WAVEFRONT_QUEUE_VERSION: u32 = 1;
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
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct QueueHeader {
    pub version: u32,
    pub count: u32,
    pub capacity: u32,
    pub overflowed: u32,
}

pub fn queue_capacity_for_resolution(
    dimensions: (u32, u32),
    render_scale: f32,
    max_capacity: usize,
) -> usize {
    let scale = render_scale.clamp(0.01, 1.0);
    let width = ((dimensions.0 as f32 * scale).ceil() as usize).max(1);
    let height = ((dimensions.1 as f32 * scale).ceil() as usize).max(1);
    width.saturating_mul(height).min(max_capacity.max(1))
}

pub fn dispatch_workgroups_for(item_count: u32, workgroup_size: u32) -> u32 {
    if item_count == 0 || workgroup_size == 0 {
        0
    } else {
        item_count.div_ceil(workgroup_size)
    }
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
