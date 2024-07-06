use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ConstantDataRw, GPUBuffer, GPUInstance,
    GPULight, GPUMaterial, GPUMesh, GPUMeshlet, GPUTexture, GPUTransform, GPUVector,
    GPUVertexAttributes, GPUVertexIndices, GPUVertexPosition, LoadOperation, Pass, RenderContext,
    RenderContextRc, RenderPass, RenderPassBeginData, RenderPassData, RenderTarget, SamplerType,
    ShaderStage, StoreOperation, TextureId, TextureView,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::{generate_random_uid, INVALID_UID};

use crate::{
    DebugPackedData, RadiancePackedData, RayPackedData, ThroughputPackedData, INSTANCE_DATA_ID,
};

pub const DEBUG_PIPELINE: &str = "pipelines/Debug.render_pipeline";
pub const DEBUG_PASS_NAME: &str = "DebugPass";

pub struct DebugPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    indices: GPUBuffer<GPUVertexIndices>,
    instances: GPUVector<GPUInstance>,
    transforms: GPUVector<GPUTransform>,
    vertices_position: GPUBuffer<GPUVertexPosition>,
    vertices_attributes: GPUBuffer<GPUVertexAttributes>,
    textures: GPUBuffer<GPUTexture>,
    materials: GPUBuffer<GPUMaterial>,
    lights: GPUBuffer<GPULight>,
    finalize_texture: TextureId,
    visibility_texture: TextureId,
    depth_texture: TextureId,
    data_buffer_0: GPUVector<RayPackedData>,
    data_buffer_1: GPUVector<RadiancePackedData>,
    data_buffer_2: GPUVector<ThroughputPackedData>,
    data_buffer_debug: GPUVector<DebugPackedData>,
}
unsafe impl Send for DebugPass {}
unsafe impl Sync for DebugPass {}

impl Pass for DebugPass {
    fn name(&self) -> &str {
        DEBUG_PASS_NAME
    }
    fn static_name() -> &'static str {
        DEBUG_PASS_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        inox_profiler::scoped_profile!("debug_pass::create");

        let data = RenderPassData {
            name: DEBUG_PASS_NAME.to_string(),
            load_color: LoadOperation::Load,
            load_depth: LoadOperation::Load,
            store_color: StoreOperation::Store,
            store_depth: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipeline: PathBuf::from(DEBUG_PIPELINE),
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
            binding_data: BindingData::new(render_context, DEBUG_PASS_NAME),
            constant_data: render_context.global_buffers().constant_data.clone(),
            meshes: render_context.global_buffers().buffer::<GPUMesh>(),
            meshlets: render_context.global_buffers().buffer::<GPUMeshlet>(),
            transforms: render_context.global_buffers().vector::<GPUTransform>(),
            instances: render_context
                .global_buffers()
                .vector_with_id::<GPUInstance>(INSTANCE_DATA_ID),
            textures: render_context.global_buffers().buffer::<GPUTexture>(),
            materials: render_context.global_buffers().buffer::<GPUMaterial>(),
            lights: render_context.global_buffers().buffer::<GPULight>(),
            indices: render_context.global_buffers().buffer::<GPUVertexIndices>(),
            vertices_position: render_context
                .global_buffers()
                .buffer::<GPUVertexPosition>(),
            vertices_attributes: render_context
                .global_buffers()
                .buffer::<GPUVertexAttributes>(),
            data_buffer_0: render_context.global_buffers().vector::<RayPackedData>(),
            data_buffer_1: render_context
                .global_buffers()
                .vector::<RadiancePackedData>(),
            data_buffer_2: render_context
                .global_buffers()
                .vector::<ThroughputPackedData>(),
            data_buffer_debug: render_context.global_buffers().vector::<DebugPackedData>(),
            finalize_texture: INVALID_UID,
            visibility_texture: INVALID_UID,
            depth_texture: INVALID_UID,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("debug_pass::init");

        if self.indices.read().unwrap().is_empty()
            || self.meshes.read().unwrap().is_empty()
            || self.meshlets.read().unwrap().is_empty()
            || self.instances.read().unwrap().is_empty()
            || self.visibility_texture.is_nil()
        {
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
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.indices.write().unwrap(),
                Some("Indices"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read | BindingFlags::Index,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.vertices_position.write().unwrap(),
                Some("Vertices Position"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read | BindingFlags::Vertex,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.vertices_attributes.write().unwrap(),
                Some("Vertices Attributes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.meshes.write().unwrap(),
                Some("Meshes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.meshlets.write().unwrap(),
                Some("Meshlets"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.instances.write().unwrap(),
                Some("Instances"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 6,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.transforms.write().unwrap(),
                Some("Transforms"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 7,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.materials.write().unwrap(),
                Some("Materials"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.textures.write().unwrap(),
                Some("Textures"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.lights.write().unwrap(),
                Some("Lights"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_texture(
                &self.visibility_texture,
                0,
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_texture(
                &self.depth_texture,
                0,
                BindingInfo {
                    group_index: 1,
                    binding_index: 4,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            );

        self.binding_data
            .add_default_sampler(
                BindingInfo {
                    group_index: 2,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
                SamplerType::Unfiltered,
            )
            .add_material_textures(BindingInfo {
                group_index: 2,
                binding_index: 1,
                stage: ShaderStage::Fragment,
                ..Default::default()
            });
        self.binding_data
            .add_buffer(
                &mut *self.data_buffer_0.write().unwrap(),
                Some("DataBuffer_0"),
                BindingInfo {
                    group_index: 3,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_1.write().unwrap(),
                Some("DataBuffer_1"),
                BindingInfo {
                    group_index: 3,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_2.write().unwrap(),
                Some("DataBuffer_2"),
                BindingInfo {
                    group_index: 3,
                    binding_index: 2,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_debug.write().unwrap(),
                Some("DataBuffer_Debug"),
                BindingInfo {
                    group_index: 3,
                    binding_index: 3,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            );
        pass.init(render_context, &mut self.binding_data, None, None);
    }
    fn update(
        &mut self,
        render_context: &RenderContext,
        surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("debug_pass::update");

        let pass = self.render_pass.get();
        let pipeline = pass.pipeline().get();
        if !pipeline.is_initialized() {
            return;
        }

        if self.indices.read().unwrap().is_empty()
            || self.meshes.read().unwrap().is_empty()
            || self.meshlets.read().unwrap().is_empty()
            || self.instances.read().unwrap().is_empty()
            || self.visibility_texture.is_nil()
        {
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
        let mut render_pass = pass.begin(&mut self.binding_data, &pipeline, render_pass_begin_data);
        {
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass,
                &render_context.webgpu.device,
                "debug_pass",
            );
            pass.draw(render_context, render_pass, 0..3, 0..1);
        }
    }
}

impl DebugPass {
    pub fn set_visibility_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.visibility_texture = *texture_id;
        self
    }
    pub fn set_depth_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.depth_texture = *texture_id;
        self
    }
    pub fn set_finalize_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.finalize_texture = *texture_id;
        self
    }
}
