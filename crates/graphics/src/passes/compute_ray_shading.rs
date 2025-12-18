use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUBuffer, GPUInstance, GPUMaterial, GPUMesh, GPUMeshlet, GPUTexture,
    GPUTransform, GPUVector, GPUVertexAttributes, GPUVertexIndices, GPUVertexPosition, Pass,
    RenderContext, RenderContextRc, ShaderStage, TextureId, TextureView, DEFAULT_HEIGHT,
    DEFAULT_WIDTH, INSTANCE_DATA_ID,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, INVALID_UID};

use crate::{IntersectionPackedData, RayPackedData};

pub const COMPUTE_RAY_SHADING_PIPELINE: &str = "pipelines/ComputeRayShading.compute_pipeline";
pub const COMPUTE_RAY_SHADING_NAME: &str = "ComputeRayShadingPass";

pub struct ComputeRayShadingPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    rays: GPUVector<RayPackedData>,
    intersections: GPUVector<IntersectionPackedData>,
    rays_next: GPUVector<RayPackedData>,
    indices: GPUBuffer<GPUVertexIndices>,
    vertices_position: GPUBuffer<GPUVertexPosition>,
    vertices_attributes: GPUBuffer<GPUVertexAttributes>,
    instances: GPUVector<GPUInstance>,
    transforms: GPUVector<GPUTransform>,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    materials: GPUBuffer<GPUMaterial>,
    textures: GPUBuffer<GPUTexture>,
    indirect_diffuse_texture: TextureId,
    indirect_specular_texture: TextureId,
}

unsafe impl Send for ComputeRayShadingPass {}
unsafe impl Sync for ComputeRayShadingPass {}

impl Pass for ComputeRayShadingPass {
    fn name(&self) -> &str {
        COMPUTE_RAY_SHADING_NAME
    }

    fn static_name() -> &'static str {
        COMPUTE_RAY_SHADING_NAME
    }

    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }

    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: COMPUTE_RAY_SHADING_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_RAY_SHADING_PIPELINE)],
        };

        let rays = render_context.global_buffers().vector::<RayPackedData>();
        let intersections = render_context
            .global_buffers()
            .vector::<IntersectionPackedData>();
        let rays_next = render_context.global_buffers().vector::<RayPackedData>();
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

        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            constant_data: render_context.global_buffers().constant_data.clone(),
            rays,
            intersections,
            rays_next,
            indices,
            vertices_position,
            vertices_attributes,
            instances,
            transforms,
            meshes,
            meshlets,
            materials,
            textures,
            binding_data: BindingData::new(render_context, COMPUTE_RAY_SHADING_NAME),
            indirect_diffuse_texture: INVALID_UID,
            indirect_specular_texture: INVALID_UID,
        }
    }

    fn init(&mut self, render_context: &RenderContext) {
        if self.indirect_diffuse_texture.is_nil() || self.indirect_specular_texture.is_nil() {
            return;
        }

        if self.instances.read().unwrap().is_empty() || self.meshlets.read().unwrap().is_empty() {
            return;
        }

        if self.rays.read().unwrap().is_empty()
            || self.intersections.read().unwrap().is_empty()
            || self.rays_next.read().unwrap().is_empty()
        {
            return;
        }

        // Group 0: Constant data + output textures
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
        // TODO: Output textures - not yet implemented in shader
        // .add_texture(
        //     &self.indirect_diffuse_texture,
        //     0,
        //     BindingInfo {
        //         group_index: 0,
        //         binding_index: 3,
        //         stage: ShaderStage::Compute,
        //         flags: BindingFlags::Write | BindingFlags::Storage,
        //         ..Default::default()
        //     },
        // )
        // .add_texture(
        //     &self.indirect_specular_texture,
        //     0,
        //     BindingInfo {
        //         group_index: 0,
        //         binding_index: 4,
        //         stage: ShaderStage::Compute,
        //         flags: BindingFlags::Write | BindingFlags::Storage,
        //         ..Default::default()
        //     },
        // );

        // Group 1: Geometry - all buffers
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
            );

        // Group 3: Ray Data only (rays, intersections, rays_next)
        self.binding_data
            .add_buffer(
                &mut *self.rays.write().unwrap(),
                Some("rays"),
                BindingInfo {
                    group_index: 3,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.intersections.write().unwrap(),
                Some("intersections"),
                BindingInfo {
                    group_index: 3,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.rays_next.write().unwrap(),
                Some("rays_next"),
                BindingInfo {
                    group_index: 3,
                    binding_index: 2, // After rays and intersections
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
        if self.indirect_diffuse_texture.is_nil() || self.indirect_specular_texture.is_nil() {
            return;
        }

        let num_rays = DEFAULT_WIDTH * DEFAULT_HEIGHT;
        let workgroup_size = 64;
        let dispatch_x = num_rays.div_ceil(workgroup_size);

        self.compute_pass.get().dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            dispatch_x,
            1,
            1,
        );
    }
}

impl ComputeRayShadingPass {
    pub fn set_indirect_diffuse_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.indirect_diffuse_texture = *texture_id;
        self
    }

    pub fn set_indirect_specular_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.indirect_specular_texture = *texture_id;
        self
    }

    pub fn rays_next(&self) -> &GPUVector<RayPackedData> {
        &self.rays_next
    }
}
