use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    DrawIndexedCommand, GPUBuffer, GPUMesh, GPUMeshlet, GPUVector, Pass, RenderContext,
    RenderContextRc, ShaderStage, TextureView,
};

use inox_core::ContextRc;

use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::MeshletLodLevel;

pub const COMMANDS_PIPELINE: &str = "pipelines/ComputeCommands.compute_pipeline";
pub const COMMANDS_PASS_NAME: &str = "CommandsPass";

pub struct CommandsPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    commands: GPUVector<DrawIndexedCommand>,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    meshlets_lod_level: GPUVector<MeshletLodLevel>,
}
unsafe impl Send for CommandsPass {}
unsafe impl Sync for CommandsPass {}

impl Pass for CommandsPass {
    fn name(&self) -> &str {
        COMMANDS_PASS_NAME
    }
    fn static_name() -> &'static str {
        COMMANDS_PASS_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let compute_data = ComputePassData {
            name: COMMANDS_PASS_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMMANDS_PIPELINE)],
        };

        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &compute_data,
                None,
            ),
            commands: render_context
                .global_buffers()
                .vector::<DrawIndexedCommand>(),
            meshes: render_context.global_buffers().buffer::<GPUMesh>(),
            meshlets: render_context.global_buffers().buffer::<GPUMeshlet>(),
            meshlets_lod_level: render_context.global_buffers().vector::<MeshletLodLevel>(),
            binding_data: BindingData::new(render_context, COMMANDS_PASS_NAME),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("compute_commands_pass::init");

        if self.meshlets_lod_level.read().unwrap().is_empty()
            || self.commands.read().unwrap().is_empty()
        {
            return;
        }

        self.binding_data
            .add_storage_buffer(
                &mut *self.meshlets.write().unwrap(),
                Some("Meshlets"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshes.write().unwrap(),
                Some("Meshes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.commands.write().unwrap(),
                Some("Commands"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Indirect,
                },
            )
            .add_storage_buffer(
                &mut *self.meshlets_lod_level.write().unwrap(),
                Some("Meshlets Lod level"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Read,
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
        inox_profiler::scoped_profile!("compute_commands_pass::update");

        if self.meshlets_lod_level.read().unwrap().is_empty() {
            return;
        }

        let num_meshlets = self.meshlets.read().unwrap().item_count();
        if num_meshlets == 0 {
            return;
        }

        let workgroup_max_size = 256;
        let workgroup_size = (num_meshlets + workgroup_max_size - 1) / workgroup_max_size;

        let pass = self.compute_pass.get();
        pass.dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            workgroup_size as u32,
            1,
            1,
        );
    }
}
