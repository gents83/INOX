use std::path::PathBuf;

use inox_core::ContextRc;
use inox_render::{
    AsBinding, BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUVector, Pass, RenderContext, RenderContextRc, ShaderStage, TextureView,
};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, generate_static_uid_from_string, Uid};

use crate::{
    queue_capacity_for_resolution, wavefront_primary_bindings, DispatchArgs, HitRecord,
    QueueHeader, RayQueueRecord, WavefrontQueueBindingIds,
};

pub const COMPUTE_WAVEFRONT_TRAVERSAL_PIPELINE: &str =
    "pipelines/ComputeWavefrontTraversal.compute_pipeline";
pub const COMPUTE_WAVEFRONT_TRAVERSAL_NAME: &str = "ComputeWavefrontTraversalPass";
pub const WAVEFRONT_HIT_QUEUE_ID: Uid = generate_static_uid_from_string("WAVEFRONT_HIT_QUEUE");
pub const WAVEFRONT_HIT_HEADER_ID: Uid = generate_static_uid_from_string("WAVEFRONT_HIT_HEADER");
pub const WAVEFRONT_SCENE_ID: Uid = generate_static_uid_from_string("WAVEFRONT_PACKED_SCENE");
const MAX_WAVEFRONT_QUEUE_CAPACITY: usize = 1_048_576;

pub struct ComputeWavefrontTraversalPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    scene_data: GPUVector<u32>,
    ray_queue: GPUVector<RayQueueRecord>,
    ray_queue_header: GPUVector<QueueHeader>,
    hit_queue: GPUVector<HitRecord>,
    hit_queue_header: GPUVector<QueueHeader>,
    dispatch_args: GPUVector<DispatchArgs>,
    pass_name: String,
}

unsafe impl Send for ComputeWavefrontTraversalPass {}
unsafe impl Sync for ComputeWavefrontTraversalPass {}

impl Pass for ComputeWavefrontTraversalPass {
    fn name(&self) -> &str {
        &self.pass_name
    }

    fn static_name() -> &'static str {
        COMPUTE_WAVEFRONT_TRAVERSAL_NAME
    }

    fn is_active(&self, _render_context: &RenderContext) -> bool {
        !self.scene_data.read().unwrap().is_empty()
    }

    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self {
        Self::create_for_queues(
            context,
            render_context,
            wavefront_primary_bindings(),
            COMPUTE_WAVEFRONT_TRAVERSAL_NAME,
        )
    }

    fn init(&mut self, render_context: &RenderContext) {
        let packed_scene = render_context.global_buffers().packed_scene_data();
        if let Err(error) = packed_scene.validate() {
            eprintln!("Skipping invalid packed scene upload: {error}");
            return;
        }
        if packed_scene.words().len() <= std::mem::size_of::<inox_render::PackedSceneHeader>() / 4 {
            return;
        }
        {
            let mut scene_data = self.scene_data.write().unwrap();
            if scene_data.as_slice() != packed_scene.words() {
                *scene_data = packed_scene.words().to_vec();
                render_context.mark_as_dirty(scene_data.buffer_id());
            }
        }
        if self.ray_queue.read().unwrap().is_empty() {
            return;
        }

        self.binding_data
            .add_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.scene_data.write().unwrap(),
                Some("Packed Scene"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.ray_queue_header.write().unwrap(),
                Some("Wavefront Ray Queue Header"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.ray_queue.write().unwrap(),
                Some("Wavefront Ray Queue"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.hit_queue_header.write().unwrap(),
                Some("Wavefront Hit Queue Header"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.hit_queue.write().unwrap(),
                Some("Wavefront Hit Queue"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
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
        {
            let mut header = self.hit_queue_header.write().unwrap();
            header[0].count = 0;
            header[0].overflowed = 0;
            header[0].capacity = self.hit_queue.read().unwrap().len() as u32;
            render_context.mark_as_dirty(header.buffer_id());
        }
        if self.scene_data.read().unwrap().is_empty()
            || self
                .ray_queue_header
                .read()
                .unwrap()
                .first()
                .is_none_or(|header| header.count == 0)
        {
            return;
        }
        let buffers = render_context.buffers();
        let dispatch_args_id = self.dispatch_args.read().unwrap().buffer_id();
        self.compute_pass.get().dispatch_indirect(
            render_context,
            &mut self.binding_data,
            command_buffer,
            &buffers,
            dispatch_args_id,
        );
    }
}

impl ComputeWavefrontTraversalPass {
    pub fn create_for_queues(
        context: &ContextRc,
        render_context: &RenderContextRc,
        ids: WavefrontQueueBindingIds,
        pass_name: &str,
    ) -> Self {
        let data = ComputePassData {
            name: pass_name.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_WAVEFRONT_TRAVERSAL_PIPELINE)],
        };
        let scene_data = render_context
            .global_buffers()
            .vector_with_id::<u32>(WAVEFRONT_SCENE_ID);
        let ray_queue = render_context
            .global_buffers()
            .vector_with_id::<RayQueueRecord>(ids.ray_queue);
        let ray_queue_header = render_context
            .global_buffers()
            .vector_with_id::<QueueHeader>(ids.ray_header);
        let hit_queue = render_context
            .global_buffers()
            .vector_with_id::<HitRecord>(ids.hit_queue);
        let hit_queue_header = render_context
            .global_buffers()
            .vector_with_id::<QueueHeader>(ids.hit_header);
        let dispatch_args = render_context
            .global_buffers()
            .vector_with_id::<DispatchArgs>(ids.dispatch_args);
        hit_queue_header
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
            binding_data: BindingData::new(render_context, pass_name),
            constant_data: render_context.global_buffers().constant_data.clone(),
            scene_data,
            ray_queue,
            ray_queue_header,
            hit_queue,
            hit_queue_header,
            dispatch_args,
            pass_name: pass_name.to_string(),
        }
    }

    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) -> &mut Self {
        let capacity = queue_capacity_for_resolution(dimensions, 1.0, MAX_WAVEFRONT_QUEUE_CAPACITY);
        {
            let mut hit_queue = self.hit_queue.write().unwrap();
            if hit_queue.len() != capacity {
                hit_queue.resize(capacity, HitRecord::default());
            }
        }
        self.hit_queue_header.write().unwrap()[0].capacity = capacity as u32;
        self
    }
}
