use std::path::PathBuf;

use inox_render::{
    platform::has_primitive_index_support, BindingData, BindingFlags, BindingInfo, CommandBuffer,
    ConstantDataRw, DrawIndexedCommand, GPUBuffer, GPUInstance, GPUMesh, GPUPrimitiveIndices,
    GPUTransform, GPUVector, GPUVertexIndices, GPUVertexPosition, Pass, RenderContext,
    RenderContextRc, RenderPass, RenderPassBeginData, RenderPassData, RenderTarget, ShaderStage,
    StoreOperation, Texture, TextureView, VextexBindingType,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::generate_random_uid;

use crate::INSTANCE_DATA_ID;

pub const VISIBILITY_BUFFER_PIPELINE: &str = "pipelines/VisibilityBuffer.render_pipeline";
pub const VISIBILITY_BUFFER_PASS_NAME: &str = "VisibilityBufferPass";

pub struct VisibilityBufferPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    indices: GPUBuffer<GPUVertexIndices>,
    primitive_indices: GPUBuffer<GPUPrimitiveIndices>,
    meshes: GPUBuffer<GPUMesh>,
    transforms: GPUVector<GPUTransform>,
    instances: GPUVector<GPUInstance>,
    commands: GPUVector<DrawIndexedCommand>,
    vertices_position: GPUBuffer<GPUVertexPosition>,
}
unsafe impl Send for VisibilityBufferPass {}
unsafe impl Sync for VisibilityBufferPass {}

impl Pass for VisibilityBufferPass {
    fn name(&self) -> &str {
        VISIBILITY_BUFFER_PASS_NAME
    }
    fn static_name() -> &'static str {
        VISIBILITY_BUFFER_PASS_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        inox_profiler::scoped_profile!("visibility_buffer_pass::create");

        let data = RenderPassData {
            name: VISIBILITY_BUFFER_PASS_NAME.to_string(),
            store_color: StoreOperation::Store,
            store_depth: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipeline: PathBuf::from(VISIBILITY_BUFFER_PIPELINE),
            ..Default::default()
        };

        Self {
            render_pass: RenderPass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            binding_data: BindingData::new(render_context, VISIBILITY_BUFFER_PASS_NAME),
            constant_data: render_context.global_buffers().constant_data.clone(),
            vertices_position: render_context
                .global_buffers()
                .buffer::<GPUVertexPosition>(),
            indices: render_context.global_buffers().buffer::<GPUVertexIndices>(),
            primitive_indices: render_context
                .global_buffers()
                .buffer::<GPUPrimitiveIndices>(),
            meshes: render_context.global_buffers().buffer::<GPUMesh>(),
            transforms: render_context.global_buffers().vector::<GPUTransform>(),
            instances: render_context
                .global_buffers()
                .vector_with_id::<GPUInstance>(INSTANCE_DATA_ID),
            commands: render_context
                .global_buffers()
                .vector::<DrawIndexedCommand>(),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("visibility_buffer_pass::init");

        let commands_count = self.commands.read().unwrap().len();
        if self.transforms.read().unwrap().is_empty() || commands_count == 0 {
            return;
        }

        let mut pass = self.render_pass.get_mut();

        self.binding_data
            .add_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Vertex,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.transforms.write().unwrap(),
                Some("Transforms"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Vertex,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .set_vertex_buffer(
                VextexBindingType::Vertex,
                &mut *self.vertices_position.write().unwrap(),
                Some("Vertices Position"),
            )
            .set_vertex_buffer(
                VextexBindingType::Instance,
                &mut *self.instances.write().unwrap(),
                Some("Instances"),
            )
            .add_buffer(
                &mut *self.commands.write().unwrap(),
                Some("Commands"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Vertex,
                    flags: BindingFlags::Storage | BindingFlags::Read | BindingFlags::Indirect,
                    count: Some(commands_count),
                },
            );

        if has_primitive_index_support() {
            self.binding_data
                .set_index_buffer(&mut *self.indices.write().unwrap(), Some("Indices"));
        } else {
            self.binding_data
                .set_index_buffer(
                    &mut *self.primitive_indices.write().unwrap(),
                    Some("Primitive Indices"),
                )
                .add_buffer(
                    &mut *self.vertices_position.write().unwrap(),
                    Some("Vertices Position"),
                    BindingInfo {
                        group_index: 0,
                        binding_index: 3,
                        stage: ShaderStage::Vertex,
                        flags: BindingFlags::Storage | BindingFlags::Read,
                        ..Default::default()
                    },
                )
                .add_buffer(
                    &mut *self.indices.write().unwrap(),
                    Some("Indices"),
                    BindingInfo {
                        group_index: 0,
                        binding_index: 4,
                        stage: ShaderStage::Vertex,
                        flags: BindingFlags::Storage | BindingFlags::Read,
                        ..Default::default()
                    },
                )
                .add_buffer(
                    &mut *self.meshes.write().unwrap(),
                    Some("Meshes"),
                    BindingInfo {
                        group_index: 0,
                        binding_index: 5,
                        stage: ShaderStage::Vertex,
                        flags: BindingFlags::Storage | BindingFlags::Read,
                        ..Default::default()
                    },
                );
        }

        let vertex_layout = GPUVertexPosition::descriptor(0);
        let instance_layout = GPUInstance::descriptor(vertex_layout.location());
        pass.init(
            render_context,
            &mut self.binding_data,
            Some(vertex_layout),
            Some(instance_layout),
        );
    }
    fn update(
        &mut self,
        render_context: &RenderContext,
        surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("visibility_buffer_pass::update");

        let pass = self.render_pass.get();
        let pipeline = pass.pipeline().get();
        if !pipeline.is_initialized() {
            return;
        }

        if self.transforms.read().unwrap().is_empty() || self.commands.read().unwrap().is_empty() {
            return;
        }

        let buffers = render_context.buffers();
        let render_targets = render_context.texture_handler().render_targets();

        let render_pass_begin_data = RenderPassBeginData {
            render_core_context: &render_context.webgpu,
            buffers: &buffers,
            render_targets: render_targets.as_slice(),
            surface_view,
            command_buffer,
        };
        #[allow(unused_mut)]
        let mut render_pass = pass.begin(&mut self.binding_data, &pipeline, render_pass_begin_data);
        {
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass,
                &render_context.webgpu.device,
                "visibility_pass",
            );
            pass.indirect_draw(render_context, &self.commands, render_pass);
        }
    }
}

impl VisibilityBufferPass {
    pub fn add_render_target(&self, texture: &Resource<Texture>) -> &Self {
        self.render_pass.get_mut().add_render_target(texture);
        self
    }
    pub fn add_depth_target(&self, texture: &Resource<Texture>) -> &Self {
        self.render_pass.get_mut().add_depth_target(texture);
        self
    }
}
