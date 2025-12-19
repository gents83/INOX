use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUBuffer, GPUInstance, GPULight, GPUMaterial, GPUMesh, GPUMeshlet, GPUTexture,
    GPUTransform, GPUVector, GPUVertexAttributes, GPUVertexIndices, GPUVertexPosition, Pass,
    RenderContext, RenderContextRc, ShaderStage, TextureId, TextureView, INSTANCE_DATA_ID,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, INVALID_UID};

use crate::RayPackedData;

pub const COMPUTE_SHADOW_GENERATION_PIPELINE: &str =
    "pipelines/ComputeShadowGeneration.compute_pipeline";
pub const COMPUTE_SHADOW_GENERATION_NAME: &str = "ComputeShadowGenerationPass";

pub struct ComputeShadowGenerationPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    shadow_rays: GPUVector<RayPackedData>,
    indices: GPUBuffer<GPUVertexIndices>,
    vertices_position: GPUBuffer<GPUVertexPosition>,
    vertices_attributes: GPUBuffer<GPUVertexAttributes>,
    instances: GPUVector<GPUInstance>,
    transforms: GPUVector<GPUTransform>,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    materials: GPUBuffer<GPUMaterial>,
    textures: GPUBuffer<GPUTexture>,
    lights: GPUBuffer<GPULight>,
    visibility_texture: TextureId,
    depth_texture: TextureId,
    dimensions: (u32, u32),
}

unsafe impl Send for ComputeShadowGenerationPass {}
unsafe impl Sync for ComputeShadowGenerationPass {}

impl Pass for ComputeShadowGenerationPass {
    fn name(&self) -> &str {
        COMPUTE_SHADOW_GENERATION_NAME
    }

    fn static_name() -> &'static str {
        COMPUTE_SHADOW_GENERATION_NAME
    }

    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }

    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: COMPUTE_SHADOW_GENERATION_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_SHADOW_GENERATION_PIPELINE)],
        };

        let shadow_rays = render_context
            .global_buffers()
            .vector_with_id::<RayPackedData>(crate::SHADOW_RAYS_ID);
        let indices = render_context.global_buffers().buffer::<GPUVertexIndices>();
        let vertices_position = render_context
            .global_buffers()
            .buffer::<GPUVertexPosition>();
        let vertices_attributes = render_context
            .global_buffers()
            .buffer::<GPUVertexAttributes>();
        let instances = render_context
            .global_buffers()
            .vector_with_id::<GPUInstance>(INSTANCE_DATA_ID);
        let transforms = render_context.global_buffers().vector::<GPUTransform>();
        let meshes = render_context.global_buffers().buffer::<GPUMesh>();
        let meshlets = render_context.global_buffers().buffer::<GPUMeshlet>();
        let materials = render_context.global_buffers().buffer::<GPUMaterial>();
        let textures = render_context.global_buffers().buffer::<GPUTexture>();
        let lights = render_context.global_buffers().buffer::<GPULight>();

        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            constant_data: render_context.global_buffers().constant_data.clone(),
            shadow_rays,
            indices,
            vertices_position,
            vertices_attributes,
            instances,
            transforms,
            meshes,
            meshlets,
            materials,
            textures,
            lights,
            binding_data: BindingData::new(render_context, COMPUTE_SHADOW_GENERATION_NAME),
            visibility_texture: INVALID_UID,
            depth_texture: INVALID_UID,
            dimensions: (0, 0),
        }
    }

    fn init(&mut self, render_context: &RenderContext) {
        if self.visibility_texture.is_nil() || self.depth_texture.is_nil() {
            return;
        }

        if self.instances.read().unwrap().is_empty() || self.meshlets.read().unwrap().is_empty() {
            return;
        }

        if self.shadow_rays.read().unwrap().is_empty() {
            return;
        }

        // Group 0: Constant data, visibility, depth
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
            .add_texture(
                &self.visibility_texture,
                0,
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_texture(
                &self.depth_texture,
                0,
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            );

        // Group 1: Geometry
        self.binding_data
            .add_buffer(
                &mut *self.indices.write().unwrap(),
                Some("indices"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.vertices_position.write().unwrap(),
                Some("vertices_positions"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.vertices_attributes.write().unwrap(),
                Some("vertices_attributes"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.instances.write().unwrap(),
                Some("instances"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.transforms.write().unwrap(),
                Some("transforms"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.meshes.write().unwrap(),
                Some("meshes"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.meshlets.write().unwrap(),
                Some("meshlets"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 6,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            );

        // Group 2: Materials and Lights
        self.binding_data
            .add_buffer(
                &mut *self.materials.write().unwrap(),
                Some("materials"),
                BindingInfo {
                    group_index: 2,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.textures.write().unwrap(),
                Some("textures"),
                BindingInfo {
                    group_index: 2,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.lights.write().unwrap(),
                Some("lights"),
                BindingInfo {
                    group_index: 2,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            );

        // Group 3: Output shadow rays
        self.binding_data.add_buffer(
            &mut *self.shadow_rays.write().unwrap(),
            Some("shadow_rays"),
            BindingInfo {
                group_index: 3,
                binding_index: 0,
                stage: ShaderStage::Compute,
                flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                ..Default::default()
            },
        );

        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &mut self.binding_data, None);
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        if self.visibility_texture.is_nil() || self.depth_texture.is_nil() {
            return;
        }

        let width = self.dimensions.0;
        let height = self.dimensions.1;

        let workgroup_size = 8;
        let dispatch_x = width.div_ceil(workgroup_size);
        let dispatch_y = height.div_ceil(workgroup_size);

        self.compute_pass.get().dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            dispatch_x,
            dispatch_y,
            1,
        );
    }
}

impl ComputeShadowGenerationPass {
    pub fn set_visibility_texture(
        &mut self,
        texture_id: TextureId,
        dimensions: (u32, u32),
    ) -> &mut Self {
        self.visibility_texture = texture_id;
        self.dimensions = dimensions;
        self
    }

    pub fn set_depth_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.depth_texture = *texture_id;
        self
    }

    pub fn shadow_rays(&self) -> &GPUVector<RayPackedData> {
        &self.shadow_rays
    }
}
