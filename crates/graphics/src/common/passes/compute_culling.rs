use std::path::PathBuf;

use crate::{
    BindingData, BindingInfo, CommandBuffer, ComputePass, ComputePassData, MeshFlags, Pass,
    RenderContext, ShaderStage,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

pub const CULLING_PIPELINE: &str = "pipelines/ComputeCulling.compute_pipeline";
pub const CULLING_PASS_NAME: &str = "CullingPass";

pub struct CullingPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
}
unsafe impl Send for CullingPass {}
unsafe impl Sync for CullingPass {}

impl Pass for CullingPass {
    fn name(&self) -> &str {
        CULLING_PASS_NAME
    }
    fn static_name() -> &'static str {
        CULLING_PASS_NAME
    }
    fn create(context: &ContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: CULLING_PASS_NAME.to_string(),
            pipelines: vec![PathBuf::from(CULLING_PIPELINE)],
        };
        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                data,
                None,
            ),
            binding_data: BindingData::default(),
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        inox_profiler::scoped_profile!("pbr_pass::init");

        if render_context.render_buffers.meshlets.is_empty() {
            return;
        }

        let mesh_flags = MeshFlags::Visible | MeshFlags::Opaque;

        if let Some(commands) = render_context.render_buffers.commands.get_mut(&mesh_flags) {
            if let Some(instances) = render_context.render_buffers.instances.get_mut(&mesh_flags) {
                self.binding_data
                    .add_uniform_buffer(
                        &render_context.core,
                        &render_context.binding_data_buffer,
                        &mut render_context.constant_data,
                        BindingInfo {
                            group_index: 0,
                            binding_index: 0,
                            stage: ShaderStage::Compute,
                            ..Default::default()
                        },
                    )
                    .add_storage_buffer(
                        &render_context.core,
                        &render_context.binding_data_buffer,
                        &mut render_context.render_buffers.meshlets,
                        BindingInfo {
                            group_index: 0,
                            binding_index: 1,
                            stage: ShaderStage::Compute,
                            ..Default::default()
                        },
                    )
                    .add_storage_buffer(
                        &render_context.core,
                        &render_context.binding_data_buffer,
                        &mut render_context.render_buffers.matrix,
                        BindingInfo {
                            group_index: 0,
                            binding_index: 2,
                            stage: ShaderStage::Compute,
                            ..Default::default()
                        },
                    )
                    .add_storage_buffer(
                        &render_context.core,
                        &render_context.binding_data_buffer,
                        instances,
                        BindingInfo {
                            group_index: 0,
                            binding_index: 3,
                            stage: ShaderStage::Compute,
                            is_instance: true,
                            ..Default::default()
                        },
                    )
                    .add_storage_buffer(
                        &render_context.core,
                        &render_context.binding_data_buffer,
                        commands,
                        BindingInfo {
                            group_index: 0,
                            binding_index: 4,
                            stage: ShaderStage::Compute,
                            read_only: false,
                            is_indirect: true,
                            ..Default::default()
                        },
                    )
                    .send_to_gpu(render_context, CULLING_PASS_NAME);

                let mut pass = self.compute_pass.get_mut();
                pass.init(render_context, &self.binding_data);
            }
        }
    }

    fn update(&mut self, render_context: &mut RenderContext, command_buffer: &mut CommandBuffer) {
        let num_meshlets = render_context.render_buffers.meshlets.item_count();
        if num_meshlets == 0 {
            return;
        }

        let mesh_flags = MeshFlags::Visible | MeshFlags::Opaque;

        if let Some(_commands) = render_context.render_buffers.commands.get_mut(&mesh_flags) {
            if let Some(_instances) = render_context.render_buffers.instances.get_mut(&mesh_flags) {
                let pass = self.compute_pass.get();

                let compute_pass = pass.begin(&self.binding_data, command_buffer);
                let num_meshlet_per_group = 32;
                let count =
                    (num_meshlets as u32 + num_meshlet_per_group - 1) / num_meshlet_per_group;
                pass.dispatch(compute_pass, count, 1, 1);
            }
        }
    }
}
