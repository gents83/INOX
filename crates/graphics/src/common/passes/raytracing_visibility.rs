use std::path::PathBuf;

use crate::{
    BHVBuffer, BindingData, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, DrawCommandType, IndicesBuffer, MeshFlags, MeshesBuffer, MeshletsBuffer,
    OutputPass, Pass, RenderContext, ShaderStage, Texture, TextureFormat, TextureId, TextureUsage,
    TextureView, VertexPositionsBuffer, VerticesBuffer,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Handle, Resource};
use inox_uid::generate_random_uid;

pub const RAYTRACING_VISIBILITY_PIPELINE: &str = "pipelines/RayTracingVisibility.compute_pipeline";
pub const RAYTRACING_VISIBILITY_NAME: &str = "RayTracingVisibilityPass";
const RAYTRACING_TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

pub struct RayTracingVisibilityPass {
    context: ContextRc,
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    meshes: MeshesBuffer,
    meshlets: MeshletsBuffer,
    tlas: BHVBuffer,
    bhv: BHVBuffer,
    vertices: VerticesBuffer,
    indices: IndicesBuffer,
    vertex_positions: VertexPositionsBuffer,
    render_target: Handle<Texture>,
}
unsafe impl Send for RayTracingVisibilityPass {}
unsafe impl Sync for RayTracingVisibilityPass {}

impl Pass for RayTracingVisibilityPass {
    fn name(&self) -> &str {
        RAYTRACING_VISIBILITY_NAME
    }
    fn static_name() -> &'static str {
        RAYTRACING_VISIBILITY_NAME
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
            name: RAYTRACING_VISIBILITY_NAME.to_string(),
            pipelines: vec![PathBuf::from(RAYTRACING_VISIBILITY_PIPELINE)],
        };

        Self {
            context: context.clone(),
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            constant_data: render_context.constant_data.clone(),
            meshes: render_context.render_buffers.meshes.clone(),
            meshlets: render_context.render_buffers.meshlets.clone(),
            tlas: render_context.render_buffers.tlas.clone(),
            bhv: render_context.render_buffers.bhv.clone(),
            vertices: render_context.render_buffers.vertices.clone(),
            indices: render_context.render_buffers.indices.clone(),
            vertex_positions: render_context.render_buffers.vertex_positions.clone(),
            binding_data: BindingData::new(render_context, RAYTRACING_VISIBILITY_NAME),
            render_target: None,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("raytracing_visibility_pass::init");

        if self.render_target.is_none() || self.meshlets.read().unwrap().is_empty() {
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
                &mut *self.indices.write().unwrap(),
                Some("Indices"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    is_index: true,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.vertices.write().unwrap(),
                Some("Vertices"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    is_vertex: true,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.vertex_positions.write().unwrap(),
                Some("VertexPositions"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshes.write().unwrap(),
                Some("Meshes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshlets.write().unwrap(),
                Some("Meshlets"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.tlas.write().unwrap(),
                Some("TLAS"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 6,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.bhv.write().unwrap(),
                Some("BHV"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 7,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_texture(
                self.render_target.as_ref().unwrap().id(),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    is_storage: true,
                    read_only: false,
                    ..Default::default()
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
        if self.render_target.is_none() || self.meshlets.read().unwrap().is_empty() {
            return;
        }
        if let Some(render_target) = &self.render_target {
            inox_profiler::scoped_profile!("raytracing_visibility_pass::update");

            let pass = self.compute_pass.get();
            let x_pixels_managed_in_shader = 16;
            let y_pixels_managed_in_shader = 16;
            let max_cluster_size = 32;
            let x = max_cluster_size
                * ((render_target.get().width() + max_cluster_size - 1) / max_cluster_size)
                / x_pixels_managed_in_shader;
            let y = max_cluster_size
                * ((render_target.get().height() + max_cluster_size - 1) / max_cluster_size)
                / y_pixels_managed_in_shader;

            let mut compute_pass =
                pass.begin(render_context, &mut self.binding_data, command_buffer);
            {
                inox_profiler::gpu_scoped_profile!(
                    &mut compute_pass,
                    &render_context.core.device,
                    "raytracing_visibility_pass",
                );
                pass.dispatch(render_context, compute_pass, x, y, 1);
            }
        }
    }
}

impl OutputPass for RayTracingVisibilityPass {
    fn render_targets_id(&self) -> Vec<TextureId> {
        [*self.render_target.as_ref().unwrap().id()].to_vec()
    }
}

impl RayTracingVisibilityPass {
    pub fn add_render_target_with_resolution(&mut self, width: u32, height: u32) -> &mut Self {
        self.render_target = Some(Texture::create_from_format(
            self.context.shared_data(),
            self.context.message_hub(),
            width,
            height,
            RAYTRACING_TEXTURE_FORMAT,
            TextureUsage::TextureBinding
                | TextureUsage::CopySrc
                | TextureUsage::CopyDst
                | TextureUsage::RenderAttachment
                | TextureUsage::StorageBinding,
        ));
        self
    }
}
