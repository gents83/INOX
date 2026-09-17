use std::path::PathBuf;

use inox_core::ContextRc;
use inox_render::{
    AsBinding, BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUVector, Pass, RenderContext, RenderContextRc, ShaderStage, TextureView,
};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, generate_static_uid_from_string, Uid};

use crate::{
    queue_capacity_for_resolution, queue_capacity_u32, DispatchArgs, HitRecord, QueueHeader,
    RadiancePackedData, RayPackedData, RayQueueRecord, WavefrontBounceBindingIds,
    MAX_WAVEFRONT_QUEUE_CAPACITY, WAVEFRONT_HIT_DISPATCH_ARGS_ID, WAVEFRONT_HIT_HEADER_ID,
    WAVEFRONT_HIT_QUEUE_ID, WAVEFRONT_PRIMARY_QUEUE_ID, WAVEFRONT_SCENE_ID,
};

pub const COMPUTE_WAVEFRONT_NEXT_BOUNCE_PIPELINE: &str =
    "pipelines/ComputeWavefrontNextBounce.compute_pipeline";
pub const COMPUTE_WAVEFRONT_NEXT_BOUNCE_NAME: &str = "ComputeWavefrontNextBouncePass";
pub const WAVEFRONT_NEXT_QUEUE_ID: Uid = generate_static_uid_from_string("WAVEFRONT_NEXT_QUEUE");
pub const WAVEFRONT_NEXT_HEADER_ID: Uid = generate_static_uid_from_string("WAVEFRONT_NEXT_HEADER");
pub const WAVEFRONT_NEXT_DISPATCH_ARGS_ID: Uid =
    generate_static_uid_from_string("WAVEFRONT_NEXT_DISPATCH_ARGS");
pub const WAVEFRONT_NEXT2_QUEUE_ID: Uid = generate_static_uid_from_string("WAVEFRONT_NEXT2_QUEUE");
pub const WAVEFRONT_NEXT2_HEADER_ID: Uid =
    generate_static_uid_from_string("WAVEFRONT_NEXT2_HEADER");
pub const WAVEFRONT_NEXT2_DISPATCH_ARGS_ID: Uid =
    generate_static_uid_from_string("WAVEFRONT_NEXT2_DISPATCH_ARGS");
pub const WAVEFRONT_NEXT3_QUEUE_ID: Uid = generate_static_uid_from_string("WAVEFRONT_NEXT3_QUEUE");
pub const WAVEFRONT_NEXT3_HEADER_ID: Uid =
    generate_static_uid_from_string("WAVEFRONT_NEXT3_HEADER");
pub const WAVEFRONT_NEXT3_DISPATCH_ARGS_ID: Uid =
    generate_static_uid_from_string("WAVEFRONT_NEXT3_DISPATCH_ARGS");

pub struct ComputeWavefrontNextBouncePass {
    compute_pass: Resource<ComputePass>,
    pass_name: String,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    scene_data: GPUVector<u32>,
    gbuffer_world: GPUVector<RayPackedData>,
    gbuffer_material: GPUVector<RadiancePackedData>,
    ray_queue: GPUVector<RayQueueRecord>,
    hit_queue: GPUVector<HitRecord>,
    hit_queue_header: GPUVector<QueueHeader>,
    next_queue: GPUVector<RayQueueRecord>,
    next_queue_header: GPUVector<QueueHeader>,
    dispatch_args: GPUVector<DispatchArgs>,
    dimensions: (u32, u32),
}

unsafe impl Send for ComputeWavefrontNextBouncePass {}
unsafe impl Sync for ComputeWavefrontNextBouncePass {}

impl Pass for ComputeWavefrontNextBouncePass {
    fn name(&self) -> &str {
        &self.pass_name
    }
    fn static_name() -> &'static str {
        COMPUTE_WAVEFRONT_NEXT_BOUNCE_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        // The hit count is GPU-written and cannot gate a later pass in this
        // command buffer.  Dispatch metadata handles the empty-queue case.
        !self.hit_queue_header.read().unwrap().is_empty()
            && !self.next_queue.read().unwrap().is_empty()
            && !self.dispatch_args.read().unwrap().is_empty()
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self {
        Self::create_for_queues(
            context,
            render_context,
            WavefrontBounceBindingIds {
                source_ray_queue: WAVEFRONT_PRIMARY_QUEUE_ID,
                source_hit_queue: WAVEFRONT_HIT_QUEUE_ID,
                source_hit_header: WAVEFRONT_HIT_HEADER_ID,
                destination_ray_queue: WAVEFRONT_NEXT_QUEUE_ID,
                destination_ray_header: WAVEFRONT_NEXT_HEADER_ID,
                dispatch_args: WAVEFRONT_HIT_DISPATCH_ARGS_ID,
            },
            COMPUTE_WAVEFRONT_NEXT_BOUNCE_NAME,
        )
    }
    fn init(&mut self, render_context: &RenderContext) {
        if self.hit_queue.read().unwrap().is_empty() || self.next_queue.read().unwrap().is_empty() {
            return;
        }
        self.binding_data.add_buffer(
            &mut *self.constant_data.write().unwrap(),
            Some("ConstantData"),
            BindingInfo {
                group_index: 0,
                binding_index: 0,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Uniform | BindingFlags::Read,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.scene_data.write().unwrap(),
            Some("Packed Scene"),
            BindingInfo {
                group_index: 0,
                binding_index: 1,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::Read,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.gbuffer_world.write().unwrap(),
            Some("GBuffer World"),
            BindingInfo {
                group_index: 1,
                binding_index: 0,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::Read,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.gbuffer_material.write().unwrap(),
            Some("GBuffer Material"),
            BindingInfo {
                group_index: 1,
                binding_index: 1,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::Read,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.ray_queue.write().unwrap(),
            Some("Ray Queue"),
            BindingInfo {
                group_index: 1,
                binding_index: 2,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::Read,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.hit_queue_header.write().unwrap(),
            Some("Hit Queue Header"),
            BindingInfo {
                group_index: 1,
                binding_index: 3,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::Read,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.hit_queue.write().unwrap(),
            Some("Hit Queue"),
            BindingInfo {
                group_index: 1,
                binding_index: 4,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::Read,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.next_queue_header.write().unwrap(),
            Some("Next Queue Header"),
            BindingInfo {
                group_index: 1,
                binding_index: 5,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.next_queue.write().unwrap(),
            Some("Next Queue"),
            BindingInfo {
                group_index: 1,
                binding_index: 6,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                ..Default::default()
            },
        );
        self.compute_pass
            .get_mut()
            .init(render_context, &mut self.binding_data, None);
    }
    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        if !self.is_active(render_context) {
            return;
        }
        {
            let mut header = self.next_queue_header.write().unwrap();
            header[0].count = 0;
            header[0].overflowed = 0;
            header[0].capacity = queue_capacity_u32(self.next_queue.read().unwrap().len());
            render_context.mark_as_dirty(header.buffer_id());
        }
        let buffers = render_context.buffers();
        let args_id = self.dispatch_args.read().unwrap().buffer_id();
        self.compute_pass.get().dispatch_indirect(
            render_context,
            &mut self.binding_data,
            command_buffer,
            &buffers,
            args_id,
        );
    }
}
impl ComputeWavefrontNextBouncePass {
    pub fn create_for_queues(
        context: &ContextRc,
        render_context: &RenderContextRc,
        ids: WavefrontBounceBindingIds,
        pass_name: &str,
    ) -> Self {
        let data = ComputePassData {
            name: pass_name.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_WAVEFRONT_NEXT_BOUNCE_PIPELINE)],
        };
        let destination_header = render_context
            .global_buffers()
            .vector_with_id::<QueueHeader>(ids.destination_ray_header);
        destination_header
            .write()
            .unwrap()
            .resize(1, QueueHeader::default());
        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            pass_name: pass_name.to_owned(),
            binding_data: BindingData::new(render_context, pass_name),
            constant_data: render_context.global_buffers().constant_data.clone(),
            scene_data: render_context
                .global_buffers()
                .vector_with_id(WAVEFRONT_SCENE_ID),
            gbuffer_world: render_context.global_buffers().vector::<RayPackedData>(),
            gbuffer_material: render_context
                .global_buffers()
                .vector::<RadiancePackedData>(),
            ray_queue: render_context
                .global_buffers()
                .vector_with_id(ids.source_ray_queue),
            hit_queue: render_context
                .global_buffers()
                .vector_with_id(ids.source_hit_queue),
            hit_queue_header: render_context
                .global_buffers()
                .vector_with_id(ids.source_hit_header),
            next_queue: render_context
                .global_buffers()
                .vector_with_id(ids.destination_ray_queue),
            next_queue_header: destination_header,
            dispatch_args: render_context
                .global_buffers()
                .vector_with_id(ids.dispatch_args),
            dimensions: (0, 0),
        }
    }

    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) -> &mut Self {
        self.dimensions = dimensions;
        let capacity = queue_capacity_for_resolution(dimensions, 1.0, MAX_WAVEFRONT_QUEUE_CAPACITY);
        self.next_queue
            .write()
            .unwrap()
            .resize(capacity, RayQueueRecord::default());
        self.next_queue_header.write().unwrap()[0].capacity = queue_capacity_u32(capacity);
        self
    }
}
