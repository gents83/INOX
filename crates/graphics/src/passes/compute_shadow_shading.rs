use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUBuffer, GPULight, GPUMaterial, GPUTexture, GPUVector, Pass, RenderContext,
    RenderContextRc, ShaderStage, TextureId, TextureView,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::{generate_random_uid, INVALID_UID};

use crate::{IntersectionPackedData, RayPackedData};

pub const COMPUTE_SHADOW_SHADING_PIPELINE: &str = "pipelines/ComputeShadowShading.compute_pipeline";
pub const COMPUTE_SHADOW_SHADING_NAME: &str = "ComputeShadowShadingPass";

pub struct ComputeShadowShadingPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    shadow_rays: GPUVector<RayPackedData>,
    shadow_intersections: GPUVector<IntersectionPackedData>,
    materials: GPUBuffer<GPUMaterial>,
    textures: GPUBuffer<GPUTexture>,
    lights: GPUBuffer<GPULight>,
    shadow_texture: TextureId,
    dimensions: (u32, u32),
}

unsafe impl Send for ComputeShadowShadingPass {}
unsafe impl Sync for ComputeShadowShadingPass {}

impl Pass for ComputeShadowShadingPass {
    fn name(&self) -> &str {
        COMPUTE_SHADOW_SHADING_NAME
    }

    fn static_name() -> &'static str {
        COMPUTE_SHADOW_SHADING_NAME
    }

    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }

    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: COMPUTE_SHADOW_SHADING_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_SHADOW_SHADING_PIPELINE)],
        };

        let shadow_rays = render_context.global_buffers().vector::<RayPackedData>();
        let shadow_intersections = render_context
            .global_buffers()
            .vector::<IntersectionPackedData>();
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
            shadow_intersections,
            materials,
            textures,
            lights,
            binding_data: BindingData::new(render_context, COMPUTE_SHADOW_SHADING_NAME),
            shadow_texture: INVALID_UID,
            dimensions: (0, 0),
        }
    }

    fn init(&mut self, render_context: &RenderContext) {
        if self.shadow_texture.is_nil() {
            return;
        }

        // Don't initialize if storage buffers are empty - wgpu will reject empty buffer bindings
        if self.shadow_rays.read().unwrap().is_empty()
            || self.shadow_intersections.read().unwrap().is_empty()
        {
            return;
        }

        // Group 0: Constant data
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

        // Group 1: Materials and Lights
        self.binding_data
            .add_buffer(
                &mut *self.materials.write().unwrap(),
                Some("materials"),
                BindingInfo {
                    group_index: 1,
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
                    group_index: 1,
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
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            );

        // Group 2: Input ray data
        self.binding_data
            .add_buffer(
                &mut *self.shadow_rays.write().unwrap(),
                Some("shadow_rays"),
                BindingInfo {
                    group_index: 2,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.shadow_intersections.write().unwrap(),
                Some("shadow_intersections"),
                BindingInfo {
                    group_index: 2,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            );

        // Group 3: Output shadow texture
        self.binding_data.add_texture(
            &self.shadow_texture,
            0,
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
        if self.shadow_texture.is_nil() {
            return;
        }

        let pass = self.compute_pass.get();
        if !pass.is_initialized() {
            return;
        }

        let width = self.dimensions.0;
        let height = self.dimensions.1;

        let workgroup_size = 8;
        let dispatch_x = width.div_ceil(workgroup_size);
        let dispatch_y = height.div_ceil(workgroup_size);

        pass.dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            dispatch_x,
            dispatch_y,
            1,
        );
    }
}

impl ComputeShadowShadingPass {
    pub fn set_shadow_texture(
        &mut self,
        texture_id: &TextureId,
        dimensions: (u32, u32),
    ) -> &mut Self {
        self.shadow_texture = *texture_id;
        self.dimensions = dimensions;
        self
    }
}
