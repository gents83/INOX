use std::path::PathBuf;

use inox_core::ContextRc;
use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUVector, Pass, RenderContext, RenderContextRc, ShaderStage, TextureView,
};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, generate_static_uid_from_string, Uid};

use crate::{
    dispatch_workgroups_for, negotiated_queue_capacity_limits, queue_capacity_for_resolution,
    queue_capacity_limits_for_extent, queue_capacity_u32, queue_requested_capacity,
    wavefront_primary_bindings, QueueHeader, RadiancePackedData, RayPackedData, RayQueueRecord,
    ThroughputPackedData, SIZE_OF_DATA_BUFFER_ELEMENT,
};

pub const COMPUTE_WAVEFRONT_PRIMARY_PIPELINE: &str =
    "pipelines/ComputeWavefrontPrimary.compute_pipeline";
pub const COMPUTE_WAVEFRONT_PRIMARY_NAME: &str = "ComputeWavefrontPrimaryPass";
pub const WAVEFRONT_PRIMARY_QUEUE_ID: Uid = generate_static_uid_from_string("WF_PRIMARY_QUEUE");
pub const WAVEFRONT_PRIMARY_HEADER_ID: Uid = generate_static_uid_from_string("WF_PRIMARY_HEADER");

pub struct ComputeWavefrontPrimaryPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    gbuffer_world: GPUVector<RayPackedData>,
    gbuffer_material: GPUVector<RadiancePackedData>,
    gbuffer_auxiliary: GPUVector<ThroughputPackedData>,
    ray_queue: GPUVector<RayQueueRecord>,
    ray_queue_header: GPUVector<QueueHeader>,
    dimensions: (u32, u32),
    capacity_limits: inox_render::QueueCapacityLimits,
}

unsafe impl Send for ComputeWavefrontPrimaryPass {}
unsafe impl Sync for ComputeWavefrontPrimaryPass {}

impl Pass for ComputeWavefrontPrimaryPass {
    fn name(&self) -> &str {
        COMPUTE_WAVEFRONT_PRIMARY_NAME
    }

    fn static_name() -> &'static str {
        COMPUTE_WAVEFRONT_PRIMARY_NAME
    }

    fn is_active(&self, _render_context: &RenderContext) -> bool {
        self.dimensions.0 != 0 && self.dimensions.1 != 0
    }

    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self {
        let data = ComputePassData {
            name: COMPUTE_WAVEFRONT_PRIMARY_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_WAVEFRONT_PRIMARY_PIPELINE)],
        };
        let primary = wavefront_primary_bindings();
        let ray_queue = render_context
            .global_buffers()
            .vector_with_id::<RayQueueRecord>(primary.ray_queue);
        let ray_queue_header = render_context
            .global_buffers()
            .vector_with_id::<QueueHeader>(primary.ray_header);
        ray_queue_header
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
            binding_data: BindingData::new(render_context, COMPUTE_WAVEFRONT_PRIMARY_NAME),
            constant_data: render_context.global_buffers().constant_data.clone(),
            gbuffer_world: render_context.global_buffers().vector::<RayPackedData>(),
            gbuffer_material: render_context
                .global_buffers()
                .vector::<RadiancePackedData>(),
            gbuffer_auxiliary: render_context
                .global_buffers()
                .vector::<ThroughputPackedData>(),
            ray_queue,
            ray_queue_header,
            dimensions: (0, 0),
            capacity_limits: negotiated_queue_capacity_limits(render_context),
        }
    }

    fn init(&mut self, render_context: &RenderContext) {
        if self.gbuffer_world.read().unwrap().is_empty()
            || self.ray_queue.read().unwrap().is_empty()
        {
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
            &mut *self.gbuffer_auxiliary.write().unwrap(),
            Some("GBuffer Auxiliary"),
            BindingInfo {
                group_index: 1,
                binding_index: 2,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::Read,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.ray_queue_header.write().unwrap(),
            Some("Wavefront Ray Queue Header"),
            BindingInfo {
                group_index: 1,
                binding_index: 3,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.ray_queue.write().unwrap(),
            Some("Wavefront Ray Queue"),
            BindingInfo {
                group_index: 1,
                binding_index: 4,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                ..Default::default()
            },
        );

        self.compute_pass
            .get_mut()
            .init(render_context, &mut self.binding_data, None);
    }

    fn reset(&mut self, render_context: &RenderContext) {
        let mut header = self.ray_queue_header.write().unwrap();
        header[0].version = crate::WAVEFRONT_QUEUE_VERSION;
        header[0].count = 0;
        header[0].overflowed = 0;
        header[0].dropped_records = 0;
        header[0].capacity = queue_capacity_u32(self.ray_queue.read().unwrap().len());
        header[0].requested_capacity =
            queue_requested_capacity(self.dimensions.0, self.dimensions.1, 1).unwrap_or(u32::MAX);
        render_context.update_buffer(&mut *header);
    }

    fn resize(&mut self, dimensions: inox_render::FrameDimensions) {
        self.set_dimensions(dimensions.render_dimensions());
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        {
            let mut header = self.ray_queue_header.write().unwrap();
            header[0].version = crate::WAVEFRONT_QUEUE_VERSION;
            header[0].count = 0;
            header[0].overflowed = 0;
            header[0].dropped_records = 0;
            header[0].capacity = queue_capacity_u32(self.ray_queue.read().unwrap().len());
            header[0].requested_capacity =
                queue_requested_capacity(self.dimensions.0, self.dimensions.1, 1)
                    .unwrap_or(u32::MAX);
            render_context.update_buffer(&mut *header);
        }
        if self.gbuffer_world.read().unwrap().is_empty()
            || self.ray_queue.read().unwrap().is_empty()
        {
            return;
        }

        let pass = self.compute_pass.get();
        pass.dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            dispatch_workgroups_for(self.dimensions.0, 8),
            dispatch_workgroups_for(self.dimensions.1, 8),
            1,
        );
    }
}

impl ComputeWavefrontPrimaryPass {
    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) -> &mut Self {
        self.dimensions = dimensions;
        let capacity_limits =
            queue_capacity_limits_for_extent(self.capacity_limits, dimensions.0, dimensions.1);
        let capacity = queue_capacity_for_resolution(
            dimensions.0,
            dimensions.1,
            std::mem::size_of::<RayQueueRecord>() as u32,
            1,
            capacity_limits,
        )
        .unwrap_or(capacity_limits.fallback_capacity.max(1)) as usize;
        let pixels = (dimensions.0 as usize).saturating_mul(dimensions.1 as usize);
        let elements = SIZE_OF_DATA_BUFFER_ELEMENT.saturating_mul(pixels);
        self.gbuffer_world
            .write()
            .unwrap()
            .resize(elements, RayPackedData::default());
        self.gbuffer_material
            .write()
            .unwrap()
            .resize(elements, RadiancePackedData::default());
        self.gbuffer_auxiliary
            .write()
            .unwrap()
            .resize(elements, ThroughputPackedData::default());
        {
            let mut queue = self.ray_queue.write().unwrap();
            if queue.len() != capacity {
                queue.resize(capacity, RayQueueRecord::default());
            }
        }
        {
            let mut header = self.ray_queue_header.write().unwrap();
            header[0].capacity = queue_capacity_u32(capacity);
            header[0].requested_capacity =
                queue_requested_capacity(dimensions.0, dimensions.1, 1).unwrap_or(u32::MAX);
        }
        self
    }
}
