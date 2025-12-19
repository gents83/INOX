use crate::{ComputeRayBufferSwapPass, ComputeRayShadingPass, ComputeRayTraversalPass};
use inox_core::ContextRc;
use inox_render::{
    CommandBuffer, ConstantDataRw, Pass, RenderContext, RenderContextRc, TextureId, TextureView,
};

/// Manager pass that orchestrates multi-bounce ray tracing
/// Creates and owns traversal/shading passes, dispatching them N times based on runtime num_bounces
pub struct RayBounceManagerPass {
    constant_data: ConstantDataRw,

    // Child passes - created and owned by manager
    traversal_pass: ComputeRayTraversalPass,
    shading_pass: ComputeRayShadingPass,
    swap_pass: ComputeRayBufferSwapPass,
}

unsafe impl Send for RayBounceManagerPass {}
unsafe impl Sync for RayBounceManagerPass {}

impl Pass for RayBounceManagerPass {
    fn name(&self) -> &str {
        "RayBounceManagerPass"
    }

    fn static_name() -> &'static str {
        "RayBounceManagerPass"
    }

    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }

    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let traversal_pass = ComputeRayTraversalPass::create(context, render_context);
        let shading_pass = ComputeRayShadingPass::create(context, render_context);
        let swap_pass = ComputeRayBufferSwapPass::create(context, render_context);
        Self {
            constant_data: render_context.global_buffers().constant_data.clone(),
            traversal_pass,
            shading_pass,
            swap_pass,
        }
    }

    fn init(&mut self, render_context: &RenderContext) {
        // Read num_bounces from ConstantData every frame - runtime configurable!
        let num_bounces = self.constant_data.read().unwrap().num_bounces();

        // Skip if no bounces configured
        if num_bounces == 0 {
            return;
        }

        self.traversal_pass.init(render_context);
        self.shading_pass.init(render_context);
        self.swap_pass.init(render_context);
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        // Read num_bounces from ConstantData every frame - runtime configurable!
        let num_bounces = self.constant_data.read().unwrap().num_bounces();

        // Skip if no bounces configured
        if num_bounces == 0 {
            return;
        }

        // Execute N bounces by calling child pass update() methods multiple times
        for bounce_index in 0..num_bounces {
            // 1. Dispatch traversal pass
            self.traversal_pass
                .update(render_context, surface_view, command_buffer);

            // 2. Dispatch shading pass
            self.shading_pass
                .update(render_context, surface_view, command_buffer);

            // 3. Swap buffers for next bounce (except on last iteration)
            // Use GPU shader to copy rays_next → rays
            if bounce_index < num_bounces - 1 {
                self.swap_pass
                    .update(render_context, surface_view, command_buffer);
            }
        }
    }
}

impl RayBounceManagerPass {
    /// Set the indirect diffuse texture ID
    pub fn set_indirect_diffuse_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.shading_pass.set_indirect_diffuse_texture(texture_id);
        self
    }

    /// Set the indirect specular texture ID
    pub fn set_indirect_specular_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.shading_pass.set_indirect_specular_texture(texture_id);
        self
    }
}
