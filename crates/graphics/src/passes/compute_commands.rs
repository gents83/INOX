use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    DrawIndexedCommand, GPUBuffer, GPUInstance, GPUMesh, GPUMeshlet, GPUVector, Pass,
    RenderContext, RenderContextRc, ShaderStage, TextureView,
};

use inox_core::ContextRc;

use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::{
    CommandsData, ACTIVE_INSTANCE_DATA_ID, COMMANDS_DATA_ID, INSTANCE_DATA_ID, MESHLETS_COUNT_ID,
};

pub const COMMANDS_PIPELINE: &str = "pipelines/ComputeCommands.compute_pipeline";
pub const COMMANDS_PASS_NAME: &str = "CommandsPass";

pub struct CommandsPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    instances: GPUVector<GPUInstance>,
    active_instances: GPUVector<GPUInstance>,
    commands_data: GPUVector<CommandsData>,
    meshlets_count: GPUVector<u32>,
    commands: GPUVector<DrawIndexedCommand>,
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
            instances: render_context
                .global_buffers()
                .vector_with_id::<GPUInstance>(INSTANCE_DATA_ID),
            active_instances: render_context
                .global_buffers()
                .vector_with_id::<GPUInstance>(ACTIVE_INSTANCE_DATA_ID),
            meshlets_count: render_context
                .global_buffers()
                .vector_with_id::<u32>(MESHLETS_COUNT_ID),
            commands_data: render_context
                .global_buffers()
                .vector_with_id::<CommandsData>(COMMANDS_DATA_ID),
            binding_data: BindingData::new(render_context, COMMANDS_PASS_NAME),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("compute_commands_pass::init");

        let commands_count = self.commands_data.read().unwrap().len();
        if self.instances.read().unwrap().is_empty() || commands_count == 0 {
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
                &mut *self.instances.write().unwrap(),
                Some("Instances"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.active_instances.write().unwrap(),
                Some("ActiveInstances"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshlets_count.write().unwrap(),
                Some("MeshletsCount"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.commands_data.write().unwrap(),
                Some("CommandsData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.commands.write().unwrap(),
                Some("Commands"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 6,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite,
                    count: Some(commands_count),
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
        inox_profiler::scoped_profile!("compute_commands_pass::update");

        if self.active_instances.read().unwrap().is_empty() {
            return;
        }

        let num = self.active_instances.read().unwrap().len();
        if num == 0 {
            return;
        }

        let workgroup_max_size = 256;
        let workgroup_size = (num + workgroup_max_size - 1) / workgroup_max_size;

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
