use std::path::PathBuf;

use crate::{
    AsBinding, BHVBuffer, BindingData, BindingFlags, BindingInfo, CommandBuffer, CommandsBuffer,
    ComputePass, ComputePassData, ConstantDataRw, CullingResults, DrawCommandType, GpuBuffer,
    MeshFlags, MeshesBuffer, MeshesFlagsBuffer, MeshletsBuffer, MeshletsCullingBuffer, OutputPass,
    Pass, RenderContext, RenderCoreContext, ShaderStage, TextureId, TextureView, ATOMIC_SIZE,
};

use inox_commands::CommandParser;
use inox_core::ContextRc;
use inox_messenger::{implement_message, Listener};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

pub const CULLING_PIPELINE: &str = "pipelines/ComputeCulling.compute_pipeline";
pub const COMPACTION_PIPELINE: &str = "pipelines/ComputeCompact.compute_pipeline";
pub const CULLING_PASS_NAME: &str = "CullingPass";
pub const COMPACTION_PASS_NAME: &str = "CompactionPass";

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone)]
pub enum CullingEvent {
    FreezeCamera,
    UnfreezeCamera,
}
implement_message!(
    CullingEvent,
    culling_event_from_command_parser,
    compare_and_discard
);

impl CullingEvent {
    fn compare_and_discard(&self, _other: &Self) -> bool {
        false
    }
    fn culling_event_from_command_parser(command_parser: CommandParser) -> Option<Self> {
        if command_parser.has("freeze_camera") {
            return Some(CullingEvent::FreezeCamera);
        } else if command_parser.has("unfreeze_camera") {
            return Some(CullingEvent::UnfreezeCamera);
        }
        None
    }
}

#[derive(Default)]
struct CullingData {
    is_dirty: bool,
    view: [[f32; 4]; 4],
    mesh_flags: u32,
    _padding: [u32; 3],
}

impl AsBinding for CullingData {
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    fn set_dirty(&mut self, is_dirty: bool) {
        self.is_dirty = is_dirty;
    }
    fn size(&self) -> u64 {
        std::mem::size_of_val(&self.view) as u64
            + std::mem::size_of_val(&self.mesh_flags) as u64
            + std::mem::size_of_val(&self._padding) as u64
    }
    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut GpuBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, &[self.view]);
        buffer.add_to_gpu_buffer(render_core_context, &[self.mesh_flags]);
        buffer.add_to_gpu_buffer(render_core_context, &[self._padding]);
    }
}

pub struct CullingPass {
    compute_pass: Resource<ComputePass>,
    compact_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    commands: CommandsBuffer,
    meshes: MeshesBuffer,
    meshes_flags: MeshesFlagsBuffer,
    meshlets: MeshletsBuffer,
    meshlets_culling: MeshletsCullingBuffer,
    blas: BHVBuffer,
    culling_data: CullingData,
    culling_result: CullingResults,
    listener: Listener,
    update_camera: bool,
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
        let compute_data = ComputePassData {
            name: CULLING_PASS_NAME.to_string(),
            pipelines: vec![PathBuf::from(CULLING_PIPELINE)],
        };
        let compact_data = ComputePassData {
            name: COMPACTION_PASS_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPACTION_PIPELINE)],
        };

        let listener = Listener::new(context.message_hub());
        listener.register::<CullingEvent>();

        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &compute_data,
                None,
            ),
            compact_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &compact_data,
                None,
            ),
            constant_data: render_context.constant_data.clone(),
            commands: render_context.render_buffers.commands.clone(),
            meshes: render_context.render_buffers.meshes.clone(),
            meshes_flags: render_context.render_buffers.meshes_flags.clone(),
            meshlets: render_context.render_buffers.meshlets.clone(),
            meshlets_culling: render_context.render_buffers.meshlets_culling.clone(),
            blas: render_context.render_buffers.blas.clone(),
            binding_data: BindingData::new(render_context, CULLING_PASS_NAME),
            culling_data: CullingData::default(),
            culling_result: render_context.render_buffers.culling_result.clone(),
            listener,
            update_camera: true,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("compute_culling_pass::init");

        self.process_messages();

        if self.meshlets.read().unwrap().is_empty() {
            return;
        }
        let mesh_flags = self.mesh_flags();

        let flags: u32 = mesh_flags.into();
        if self.culling_data.mesh_flags != flags {
            self.culling_data.mesh_flags = flags;
            self.culling_data.set_dirty(true);
        }

        if self.update_camera {
            let view = self.constant_data.read().unwrap().view();
            if self.culling_data.view != view {
                self.culling_data.view = view;
                self.culling_data.set_dirty(true);
            }
        }

        let draw_command_type = self.draw_commands_type();

        if let Some(commands) = self.commands.write().unwrap().get_mut(&mesh_flags) {
            let commands = commands.map.get_mut(&draw_command_type).unwrap();
            if commands.commands.is_empty() {
                return;
            }
            commands.counter.count = 0;

            let num_meshlets = self.meshlets.read().unwrap().item_count();
            let count = ((num_meshlets as u32 + ATOMIC_SIZE - 1) / ATOMIC_SIZE) as usize;
            self.culling_result.write().unwrap().set(vec![0u32; count]);

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
                    &mut self.culling_data,
                    Some("CullingData"),
                    BindingInfo {
                        group_index: 0,
                        binding_index: 1,
                        stage: ShaderStage::Compute,
                        ..Default::default()
                    },
                )
                .add_storage_buffer(
                    &mut *self.meshlets.write().unwrap(),
                    Some("Meshlets"),
                    BindingInfo {
                        group_index: 0,
                        binding_index: 2,
                        stage: ShaderStage::Compute,
                        ..Default::default()
                    },
                )
                .add_storage_buffer(
                    &mut *self.meshlets_culling.write().unwrap(),
                    Some("MeshletsCulling"),
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
                    &mut *self.blas.write().unwrap(),
                    Some("BLAS BHV"),
                    BindingInfo {
                        group_index: 0,
                        binding_index: 5,
                        stage: ShaderStage::Compute,
                        ..Default::default()
                    },
                )
                .add_storage_buffer(
                    &mut *self.meshes_flags.write().unwrap(),
                    Some("MeshesFlags"),
                    BindingInfo {
                        group_index: 0,
                        binding_index: 6,
                        stage: ShaderStage::Compute,
                        ..Default::default()
                    },
                )
                .add_storage_buffer(
                    &mut commands.counter,
                    Some("Counter"),
                    BindingInfo {
                        group_index: 1,
                        binding_index: 0,
                        stage: ShaderStage::Compute,
                        flags: BindingFlags::ReadWrite | BindingFlags::Indirect,
                    },
                )
                .add_storage_buffer(
                    &mut commands.commands,
                    Some("Commands"),
                    BindingInfo {
                        group_index: 1,
                        binding_index: 1,
                        stage: ShaderStage::Compute,
                        flags: BindingFlags::ReadWrite | BindingFlags::Indirect,
                    },
                )
                .add_storage_buffer(
                    &mut *self.culling_result.write().unwrap(),
                    Some("CullingResult"),
                    BindingInfo {
                        group_index: 1,
                        binding_index: 2,
                        stage: ShaderStage::Compute,
                        flags: BindingFlags::ReadWrite | BindingFlags::Indirect,
                    },
                );

            let mut pass = self.compute_pass.get_mut();
            pass.init(render_context, &mut self.binding_data);

            let mut pass = self.compact_pass.get_mut();
            pass.init(render_context, &mut self.binding_data);
        }
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        let num_meshlets = self.meshlets.read().unwrap().item_count();
        if num_meshlets == 0 {
            return;
        }

        let mesh_flags = self.mesh_flags();

        if let Some(commands) = self.commands.write().unwrap().get_mut(&mesh_flags) {
            let commands = commands.map.get(&self.draw_commands_type()).unwrap();
            if commands.commands.is_empty() {
                return;
            }
            let count = (num_meshlets as u32 + ATOMIC_SIZE - 1) / ATOMIC_SIZE;

            let pass = self.compute_pass.get();
            pass.dispatch(render_context, &mut self.binding_data, command_buffer, count, 1, 1);
            pass.dispatch(render_context, &mut self.binding_data, command_buffer, count, 1, 1);
        }
    }
}

impl OutputPass for CullingPass {
    fn render_targets_id(&self) -> Option<Vec<TextureId>> {
        None
    }
    fn depth_target_id(&self) -> Option<TextureId> {
        None
    }
}

impl CullingPass {
    fn process_messages(&mut self) {
        self.listener
            .process_messages(|event: &CullingEvent| match event {
                CullingEvent::FreezeCamera => {
                    self.update_camera = false;
                }
                CullingEvent::UnfreezeCamera => {
                    self.update_camera = true;
                }
            });
    }
}
