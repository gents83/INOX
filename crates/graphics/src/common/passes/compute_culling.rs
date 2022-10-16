use std::path::PathBuf;

use crate::{
    declare_as_binding_vector, AsBinding, BindingData, BindingInfo, CommandBuffer, CommandsBuffer,
    ComputePass, ComputePassData, ConstantDataRw, DrawCommandType, GpuBuffer, MeshFlags,
    MeshesBuffer, MeshletsAABBsBuffer, MeshletsBuffer, Pass, RenderContext, RenderCoreContext,
    ShaderStage, TextureView,
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

const NUM_COMMANDS_PER_GROUP: u32 = 32;

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
}

impl AsBinding for CullingData {
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    fn set_dirty(&mut self, is_dirty: bool) {
        self.is_dirty = is_dirty;
    }
    fn size(&self) -> u64 {
        std::mem::size_of_val(&self.view) as _
    }
    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut GpuBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, &[self.view]);
    }
}

declare_as_binding_vector!(VecVisibleDrawData, u32);

pub struct CullingPass {
    compute_pass: Resource<ComputePass>,
    compact_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    commands: CommandsBuffer,
    meshes: MeshesBuffer,
    meshlets: MeshletsBuffer,
    meshlets_aabb: MeshletsAABBsBuffer,
    culling_data: CullingData,
    visible_draw_data: VecVisibleDrawData,
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
            meshlets: render_context.render_buffers.meshlets.clone(),
            meshlets_aabb: render_context.render_buffers.meshlets_aabb.clone(),
            binding_data: BindingData::new(render_context, CULLING_PASS_NAME),
            culling_data: CullingData::default(),
            visible_draw_data: VecVisibleDrawData::default(),
            listener,
            update_camera: true,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("pbr_pass::init");

        self.process_messages();

        if self.meshlets.read().unwrap().is_empty() {
            return;
        }

        if self.update_camera {
            let view = self.constant_data.read().unwrap().view();
            if self.culling_data.view != view {
                self.culling_data.view = view;
                self.culling_data.set_dirty(true);
            }
        }

        let mesh_flags = self.mesh_flags();
        let draw_command_type = self.draw_commands_type();

        if let Some(commands) = self.commands.write().unwrap().get_mut(&mesh_flags) {
            let commands = commands.map.get_mut(&draw_command_type).unwrap();
            if commands.commands.is_empty() {
                return;
            }
            commands.counter.count = 0;

            let num_meshlets = self.meshlets.read().unwrap().item_count();
            let count = ((num_meshlets as u32 + NUM_COMMANDS_PER_GROUP - 1)
                / NUM_COMMANDS_PER_GROUP) as usize;
            if self.visible_draw_data.data.len() < count {
                self.visible_draw_data.set(vec![0u32; count]);
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
                    &mut *self.meshes.write().unwrap(),
                    Some("Meshes"),
                    BindingInfo {
                        group_index: 0,
                        binding_index: 3,
                        stage: ShaderStage::Compute,
                        ..Default::default()
                    },
                )
                .add_storage_buffer(
                    &mut *self.meshlets_aabb.write().unwrap(),
                    Some("MeshletsAABB"),
                    BindingInfo {
                        group_index: 0,
                        binding_index: 4,
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
                        read_only: false,
                        is_indirect: true,
                        ..Default::default()
                    },
                )
                .add_storage_buffer(
                    &mut commands.commands,
                    Some("Commands"),
                    BindingInfo {
                        group_index: 1,
                        binding_index: 1,
                        stage: ShaderStage::Compute,
                        read_only: false,
                        is_indirect: true,
                        ..Default::default()
                    },
                )
                .add_storage_buffer(
                    &mut self.visible_draw_data,
                    Some("VisibleDrawData"),
                    BindingInfo {
                        group_index: 1,
                        binding_index: 2,
                        stage: ShaderStage::Compute,
                        read_only: false,
                        is_indirect: true,
                        ..Default::default()
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
            let count = (num_meshlets as u32 + NUM_COMMANDS_PER_GROUP - 1) / NUM_COMMANDS_PER_GROUP;

            let pass = self.compute_pass.get();
            let mut compute_pass =
                pass.begin(render_context, &mut self.binding_data, command_buffer);
            {
                inox_profiler::gpu_scoped_profile!(
                    &mut compute_pass,
                    &render_context.core.device,
                    "compute_culling_pass",
                );
                pass.dispatch(render_context, compute_pass, count, 1, 1);
            }

            let pass = self.compact_pass.get();
            let mut compact_pass =
                pass.begin(render_context, &mut self.binding_data, command_buffer);
            {
                inox_profiler::gpu_scoped_profile!(
                    &mut compact_pass,
                    &render_context.core.device,
                    "compute_compact_pass",
                );
                pass.dispatch(render_context, compact_pass, count, 1, 1);
            }
        }
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
