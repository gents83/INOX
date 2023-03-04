use std::path::PathBuf;

use crate::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, DrawCommandType, DrawRay, MeshFlags, OutputPass, Pass, RaysBuffer,
    RenderContext, ShaderStage, Texture, TextureId, TextureView,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, generate_static_uid_from_string, Uid, INVALID_UID};

pub const RAYTRACING_GENERATE_RAY_PIPELINE: &str =
    "pipelines/RayTracingGenerateRay.compute_pipeline";
pub const RAYTRACING_GENERATE_RAY_NAME: &str = "RayTracingGenerateRayPass";

const RAYS_UID: Uid = generate_static_uid_from_string("RAYS");

pub struct RayTracingGenerateRayPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    render_target_id: TextureId,
    width: u32,
    height: u32,
    rays: RaysBuffer,
}
unsafe impl Send for RayTracingGenerateRayPass {}
unsafe impl Sync for RayTracingGenerateRayPass {}

impl Pass for RayTracingGenerateRayPass {
    fn name(&self) -> &str {
        RAYTRACING_GENERATE_RAY_NAME
    }
    fn static_name() -> &'static str {
        RAYTRACING_GENERATE_RAY_NAME
    }
    fn is_active(&self, render_context: &RenderContext) -> bool {
        render_context.has_commands(&self.draw_commands_type(), &self.mesh_flags())
    }
    fn mesh_flags(&self) -> MeshFlags {
        MeshFlags::Visible | MeshFlags::Opaque
    }
    fn draw_commands_type(&self) -> DrawCommandType {
        DrawCommandType::PerMeshlet
    }
    fn create(context: &ContextRc, render_context: &RenderContext) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: RAYTRACING_GENERATE_RAY_NAME.to_string(),
            pipelines: vec![PathBuf::from(RAYTRACING_GENERATE_RAY_PIPELINE)],
        };

        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            constant_data: render_context.constant_data.clone(),
            binding_data: BindingData::new(render_context, RAYTRACING_GENERATE_RAY_NAME),
            rays: render_context.render_buffers.rays.clone(),
            width: 0,
            height: 0,
            render_target_id: INVALID_UID,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("raytracing_generate_ray_pass::init");

        if self.render_target_id.is_nil() {
            return;
        }

        self.binding_data
            .add_uniform_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.rays.write().unwrap(),
                Some("Rays"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite,
                },
            )
            .add_texture(
                &self.render_target_id,
                BindingInfo {
                    group_index: 2,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Read | BindingFlags::Storage,
                },
            );

        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &mut self.binding_data);
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        if self.render_target_id.is_nil() || self.rays.read().unwrap().is_empty() {
            return;
        }

        inox_profiler::scoped_profile!("raytracing_generate_ray_pass::update");

        let pass = self.compute_pass.get();
        let x_pixels_managed_in_shader = 16;
        let y_pixels_managed_in_shader = 16;
        let max_cluster_size = x_pixels_managed_in_shader.max(y_pixels_managed_in_shader);
        let x = (max_cluster_size * ((self.width + max_cluster_size - 1) / max_cluster_size))
            / x_pixels_managed_in_shader;
        let y = (max_cluster_size * ((self.height + max_cluster_size - 1) / max_cluster_size))
            / y_pixels_managed_in_shader;

        let mut compute_pass = pass.begin(render_context, &mut self.binding_data, command_buffer);
        {
            inox_profiler::gpu_scoped_profile!(
                &mut compute_pass,
                &render_context.core.device,
                "raytracing_generate_ray_pass",
            );
            pass.dispatch(render_context, compute_pass, x, y, 1);
        }
    }
}

impl OutputPass for RayTracingGenerateRayPass {
    fn render_targets_id(&self) -> Vec<TextureId> {
        [self.render_target_id].to_vec()
    }
}

impl RayTracingGenerateRayPass {
    pub fn use_render_target(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.render_target_id = *texture.id();
        self.width = texture.get().width();
        self.height = texture.get().height();
        {
            let mut rays = self.rays.write().unwrap();
            let ray_data = vec![DrawRay::default(); (self.width * self.height) as usize];
            rays.allocate(&RAYS_UID, ray_data.as_ref());
        }
        self
    }
}
