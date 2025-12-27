use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ConstantDataRw, GPUBuffer, GPUInstance,
    GPULight, GPUMaterial, GPUMesh, GPUMeshlet, GPUTexture, GPUTransform, GPUVector,
    GPUVertexAttributes, GPUVertexIndices, GPUVertexPosition, Pass, RenderContext, RenderContextRc,
    RenderPass, RenderPassBeginData, RenderPassData, RenderTarget, ShaderStage, StoreOperation,
    Texture, TextureView, INSTANCE_DATA_ID,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Handle, Resource, ResourceTrait};
use inox_uid::generate_random_uid;

pub const DEBUG_PIPELINE: &str = "pipelines/Debug.render_pipeline";
pub const DEBUG_PASS_NAME: &str = "DebugPass";

pub struct DebugPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    indices: GPUBuffer<GPUVertexIndices>,
    vertices_positions: GPUBuffer<GPUVertexPosition>,
    vertices_attributes: GPUBuffer<GPUVertexAttributes>,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    instances: GPUVector<GPUInstance>,
    transforms: GPUVector<GPUTransform>,
    materials: GPUBuffer<GPUMaterial>,
    textures: GPUBuffer<GPUTexture>,
    lights: GPUBuffer<GPULight>,
    visibility_texture: Handle<Texture>,
    depth_texture: Handle<Texture>,
    direct_texture: Handle<Texture>,
    indirect_diffuse_texture: Handle<Texture>,
    indirect_specular_texture: Handle<Texture>,
    shadow_texture: Handle<Texture>,
    ao_texture: Handle<Texture>,
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
        let data = RenderPassData {
            name: DEBUG_PASS_NAME.to_string(),
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
            constant_data: render_context.global_buffers().constant_data.clone(),
            binding_data: BindingData::new(render_context, DEBUG_PASS_NAME),
            indices: render_context.global_buffers().buffer::<GPUVertexIndices>(),
            vertices_positions: render_context
                .global_buffers()
                .buffer::<GPUVertexPosition>(),
            vertices_attributes: render_context
                .global_buffers()
                .buffer::<GPUVertexAttributes>(),
            meshes: render_context.global_buffers().buffer::<GPUMesh>(),
            meshlets: render_context.global_buffers().buffer::<GPUMeshlet>(),
            instances: render_context
                .global_buffers()
                .vector_with_id::<GPUInstance>(INSTANCE_DATA_ID),
            transforms: render_context.global_buffers().vector::<GPUTransform>(),
            lights: render_context.global_buffers().buffer::<GPULight>(),
            materials: render_context.global_buffers().buffer::<GPUMaterial>(),
            textures: render_context.global_buffers().buffer::<GPUTexture>(),
            visibility_texture: None,
            depth_texture: None,
            direct_texture: None,
            indirect_diffuse_texture: None,
            indirect_specular_texture: None,
            shadow_texture: None,
            ao_texture: None,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        if self.indices.read().unwrap().is_empty()
            || self.meshes.read().unwrap().is_empty()
            || self.meshlets.read().unwrap().is_empty()
            || self.instances.read().unwrap().is_empty()
            || self.visibility_texture.is_none()
            || self.direct_texture.is_none()
            || self.indirect_diffuse_texture.is_none()
            || self.indirect_specular_texture.is_none()
            || self.shadow_texture.is_none()
            || self.ao_texture.is_none()
        {
            return;
        }

        inox_profiler::scoped_profile!("debug_pass::init");

        let mut pass = self.render_pass.get_mut();
        pass.remove_all_render_targets();

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
                    flags: BindingFlags::Read | BindingFlags::Storage,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.vertices_positions.write().unwrap(),
                Some("VerticesPositions"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Read | BindingFlags::Storage,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.vertices_attributes.write().unwrap(),
                Some("VerticesAttributes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Read | BindingFlags::Storage,
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
                    flags: BindingFlags::Read | BindingFlags::Storage,
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
                    flags: BindingFlags::Read | BindingFlags::Storage,
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
                    flags: BindingFlags::Read | BindingFlags::Storage,
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
                    flags: BindingFlags::Read | BindingFlags::Storage,
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
                    flags: BindingFlags::Uniform | BindingFlags::Read,
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
                    flags: BindingFlags::Uniform | BindingFlags::Read,
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
                    flags: BindingFlags::Read | BindingFlags::Uniform,
                    ..Default::default()
                },
            )
            .add_texture(
                self.visibility_texture.as_ref().unwrap().id(),
                0,
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_texture(
                self.depth_texture.as_ref().unwrap().id(),
                0,
                BindingInfo {
                    group_index: 1,
                    binding_index: 4,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_texture(
                self.direct_texture.as_ref().unwrap().id(),
                0,
                BindingInfo {
                    group_index: 1,
                    binding_index: 5,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_texture(
                self.shadow_texture.as_ref().unwrap().id(),
                0,
                BindingInfo {
                    group_index: 1,
                    binding_index: 6,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_texture(
                self.ao_texture.as_ref().unwrap().id(),
                0,
                BindingInfo {
                    group_index: 1,
                    binding_index: 7,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_texture(
                self.indirect_diffuse_texture.as_ref().unwrap().id(),
                0,
                BindingInfo {
                    group_index: 2,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_texture(
                self.indirect_specular_texture.as_ref().unwrap().id(),
                0,
                BindingInfo {
                    group_index: 2,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Read,
                    ..Default::default()
                },
            );

        // Group 3: Texture sampling (sampler + texture arrays) - required by shader
        self.binding_data
            .add_default_sampler(
                inox_render::BindingInfo {
                    group_index: 3,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
                inox_render::SamplerType::Unfiltered,
            )
            .add_material_textures(inox_render::BindingInfo {
                group_index: 3,
                binding_index: 1,
                stage: ShaderStage::Fragment,
                ..Default::default()
            });

        pass.init(render_context, &mut self.binding_data, None, None);
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        // Check Flags: If no debug flag is set, return immediately.
        let flags = self.constant_data.read().unwrap().flags();
        if flags == 0 {
            return;
        }

        if self.indices.read().unwrap().is_empty()
            || self.meshes.read().unwrap().is_empty()
            || self.meshlets.read().unwrap().is_empty()
            || self.instances.read().unwrap().is_empty()
            || self.visibility_texture.is_none()
            || self.direct_texture.is_none()
            || self.indirect_diffuse_texture.is_none()
            || self.indirect_specular_texture.is_none()
            || self.shadow_texture.is_none()
            || self.ao_texture.is_none()
        {
            return;
        }

        inox_profiler::scoped_profile!("debug_pass::update");

        let pass = self.render_pass.get();
        let pipeline = pass.pipeline().get();
        if !pipeline.is_initialized() {
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
                "debug_pass",
            );
            pass.draw(render_context, render_pass, 0..3, 0..1);
        }
    }
}

impl DebugPass {
    pub fn set_direct_texture(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.direct_texture = Some(texture.clone());
        self
    }
    pub fn set_indirect_diffuse_texture(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.indirect_diffuse_texture = Some(texture.clone());
        self
    }
    pub fn set_indirect_specular_texture(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.indirect_specular_texture = Some(texture.clone());
        self
    }
    pub fn set_shadow_texture(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.shadow_texture = Some(texture.clone());
        self
    }
    pub fn set_ao_texture(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.ao_texture = Some(texture.clone());
        self
    }
    pub fn set_visibility_texture(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.visibility_texture = Some(texture.clone());
        self
    }
    pub fn set_depth_texture(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.depth_texture = Some(texture.clone());
        self
    }
}
