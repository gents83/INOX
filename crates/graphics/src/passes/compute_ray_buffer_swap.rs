use crate::RayPackedData;
use inox_core::ContextRc;
use inox_render::{
    BindingData, CommandBuffer, ComputePass, GPUVector, Pass, RenderContext, RenderContextRc,
    TextureView, DEFAULT_HEIGHT, DEFAULT_WIDTH,
};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;
use std::path::PathBuf;

const COMPUTE_RAY_BUFFER_SWAP_NAME: &str = "ComputeRayBufferSwap";
const COMPUTE_RAY_BUFFER_SWAP_PIPELINE: &str = "pipelines/ComputeRayBufferSwap.compute_pipeline";

pub struct ComputeRayBufferSwapPass {
    compute_pass: Resource<ComputePass>,
    rays_next: GPUVector<RayPackedData>,
    rays: GPUVector<RayPackedData>,
    binding_data: BindingData,
}

unsafe impl Send for ComputeRayBufferSwapPass {}
unsafe impl Sync for ComputeRayBufferSwapPass {}

impl Pass for ComputeRayBufferSwapPass {
    fn name(&self) -> &str {
        COMPUTE_RAY_BUFFER_SWAP_NAME
    }

    fn static_name() -> &'static str {
        COMPUTE_RAY_BUFFER_SWAP_NAME
    }

    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }

    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = inox_render::ComputePassData {
            name: COMPUTE_RAY_BUFFER_SWAP_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_RAY_BUFFER_SWAP_PIPELINE)],
        };

        let rays_next = render_context
            .global_buffers()
            .vector_with_id::<RayPackedData>(crate::BOUNCE_RAYS_NEXT_ID);
        let rays = render_context
            .global_buffers()
            .vector_with_id::<RayPackedData>(crate::BOUNCE_RAYS_ID);

        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            rays_next,
            rays,
            binding_data: BindingData::new(render_context, COMPUTE_RAY_BUFFER_SWAP_NAME),
        }
    }

    fn init(&mut self, render_context: &RenderContext) {
        // Skip if buffers are empty
        if self.rays.read().unwrap().is_empty() || self.rays_next.read().unwrap().is_empty() {
            return;
        }

        // Group 0: Buffers
        // @binding(0) rays_next (read)
        self.binding_data.add_buffer(
            &mut *self.rays_next.write().unwrap(),
            Some("rays_next"),
            inox_render::BindingInfo {
                group_index: 0,
                binding_index: 0,
                stage: inox_render::ShaderStage::Compute,
                flags: inox_render::BindingFlags::Storage | inox_render::BindingFlags::Read,
                ..Default::default()
            },
        );

        // @binding(1) rays (read_write)
        self.binding_data.add_buffer(
            &mut *self.rays.write().unwrap(),
            Some("rays"),
            inox_render::BindingInfo {
                group_index: 0,
                binding_index: 1,
                stage: inox_render::ShaderStage::Compute,
                flags: inox_render::BindingFlags::Storage | inox_render::BindingFlags::ReadWrite,
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
