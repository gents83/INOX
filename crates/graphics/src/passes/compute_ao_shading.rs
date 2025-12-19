use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUVector, Pass, RenderContext, RenderContextRc, ShaderStage, TextureId,
    TextureView,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::{generate_random_uid, INVALID_UID};

use crate::{IntersectionPackedData, RayPackedData};

pub const COMPUTE_AO_SHADING_PIPELINE: &str = "pipelines/ComputeAOShading.compute_pipeline";
pub const COMPUTE_AO_SHADING_NAME: &str = "ComputeAOShadingPass";

pub struct ComputeAOShadingPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    ao_rays: GPUVector<RayPackedData>,
    ao_intersections: GPUVector<IntersectionPackedData>,
    ao_texture: TextureId,
    dimensions: (u32, u32),
}

unsafe impl Send for ComputeAOShadingPass {}
unsafe impl Sync for ComputeAOShadingPass {}

impl Pass for ComputeAOShadingPass {
    fn name(&self) -> &str {
        COMPUTE_AO_SHADING_NAME
    }

    fn static_name() -> &'static str {
        COMPUTE_AO_SHADING_NAME
    }

    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }

    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: COMPUTE_AO_SHADING_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_AO_SHADING_PIPELINE)],
        };

        let ao_rays = render_context
            .global_buffers()
            .vector_with_id::<RayPackedData>(crate::AO_RAYS_ID);
        let ao_intersections = render_context
            .global_buffers()
            .vector_with_id::<IntersectionPackedData>(crate::AO_INTERSECTIONS_ID);

        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            constant_data: render_context.global_buffers().constant_data.clone(),
            ao_rays,
            ao_intersections,
            binding_data: BindingData::new(render_context, COMPUTE_AO_SHADING_NAME),
            ao_texture: INVALID_UID,
            dimensions: (0, 0),
        }
    }

    fn init(&mut self, render_context: &RenderContext) {
        if self.ao_texture.is_nil() {
            return;
        }

        // Don't initialize if storage buffers are empty - wgpu will reject empty buffer bindings
        if self.ao_rays.read().unwrap().is_empty()
            || self.ao_intersections.read().unwrap().is_empty()
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

        // Group 1: Input ray data
        self.binding_data
            .add_buffer(
                &mut *self.ao_rays.write().unwrap(),
                Some("ao_rays"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.ao_intersections.write().unwrap(),
                Some("ao_intersections"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            );

        // Group 2: Output AO texture
        self.binding_data.add_texture(
            &self.ao_texture,
            0,
            BindingInfo {
                group_index: 2,
                binding_index: 0,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Write | BindingFlags::Storage, // STORE only, not LOAD | STORE
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
        if self.ao_texture.is_nil() {
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

impl ComputeAOShadingPass {
    pub fn set_ao_texture(&mut self, texture_id: &TextureId, dimensions: (u32, u32)) -> &mut Self {
        self.ao_texture = *texture_id;
        self.dimensions = dimensions;
        self
    }
}
