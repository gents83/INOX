use std::path::PathBuf;

use crate::{
    AsBinding, BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, DrawCommandType, DrawRay, GpuBuffer, MeshFlags, Pass, RaysBuffer,
    RenderContext, RenderCoreContext, ShaderStage, TextureView,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, generate_static_uid_from_string, Uid};

pub const COMPUTE_RAYTRACING_GENERATE_RAY_PIPELINE: &str =
    "pipelines/ComputeRayTracingGenerateRay.compute_pipeline";
pub const COMPUTE_RAYTRACING_GENERATE_RAY_NAME: &str = "ComputeRayTracingGenerateRayPass";

const RAYS_UID: Uid = generate_static_uid_from_string("RAYS");

#[derive(Default)]
struct Data {
    width: u32,
    height: u32,
    is_dirty: bool,
}

impl AsBinding for Data {
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    fn set_dirty(&mut self, is_dirty: bool) {
        self.is_dirty = is_dirty;
    }
    fn size(&self) -> u64 {
        std::mem::size_of_val(&self.width) as u64 + std::mem::size_of_val(&self.height) as u64
    }
    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut GpuBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, &[self.width]);
        buffer.add_to_gpu_buffer(render_core_context, &[self.height]);
    }
}

pub struct ComputeRayTracingGenerateRayPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    data: Data,
    rays: RaysBuffer,
}
unsafe impl Send for ComputeRayTracingGenerateRayPass {}
unsafe impl Sync for ComputeRayTracingGenerateRayPass {}

impl Pass for ComputeRayTracingGenerateRayPass {
    fn name(&self) -> &str {
        COMPUTE_RAYTRACING_GENERATE_RAY_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_RAYTRACING_GENERATE_RAY_NAME
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
            name: COMPUTE_RAYTRACING_GENERATE_RAY_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_RAYTRACING_GENERATE_RAY_PIPELINE)],
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
            binding_data: BindingData::new(render_context, COMPUTE_RAYTRACING_GENERATE_RAY_NAME),
            rays: render_context.render_buffers.rays.clone(),
            data: Data::default(),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("raytracing_generate_ray_pass::init");

        if self.data.width == 0 || self.data.height == 0 {
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
            .add_uniform_buffer(
                &mut self.data,
                Some("Data"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
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
        if self.data.width == 0 || self.data.height == 0 {
            return;
        }

        inox_profiler::scoped_profile!("raytracing_generate_ray_pass::update");

        let pass = self.compute_pass.get();
        let x_pixels_managed_in_shader = 16;
        let y_pixels_managed_in_shader = 16;
        let max_cluster_size = x_pixels_managed_in_shader.max(y_pixels_managed_in_shader);
        let x = (max_cluster_size * ((self.data.width + max_cluster_size - 1) / max_cluster_size))
            / x_pixels_managed_in_shader;
        let y = (max_cluster_size * ((self.data.height + max_cluster_size - 1) / max_cluster_size))
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

impl ComputeRayTracingGenerateRayPass {
    pub fn set_dimensions(&mut self, width: u32, height: u32) -> &mut Self {
        self.data.width = width;
        self.data.height = height;
        self.data.set_dirty(true);
        {
            let mut rays = self.rays.write().unwrap();
            let ray_data = vec![DrawRay::default(); (self.data.width * self.data.height) as usize];
            rays.allocate(&RAYS_UID, ray_data.as_ref());
        }
        self
    }
}
