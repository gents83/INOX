use std::path::PathBuf;

use crate::{
    ArrayU32, AsBinding, BVHBuffer, BindingData, BindingFlags, BindingInfo, CommandBuffer,
    ComputePass, ComputePassData, ConstantDataRw, DrawCommandType, DrawCommandsBuffer, GpuBuffer,
    Mesh, MeshFlags, MeshesBuffer, MeshletsBuffer, Pass, RenderContext, RenderContextRc,
    ShaderStage, TextureView, ATOMIC_SIZE,
};

use inox_commands::CommandParser;
use inox_core::ContextRc;
use inox_messenger::{implement_message, Listener};
use inox_resources::{DataTypeResource, DataTypeResourceEvent, Resource, ResourceEvent};
use inox_uid::generate_random_uid;

pub const CULLING_PIPELINE: &str = "pipelines/ComputeCulling.compute_pipeline";
pub const CULLING_PASS_NAME: &str = "CullingPass";

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
    lod0_meshlets_count: u32,
    _padding1: u32,
    _padding2: u32,
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
            + std::mem::size_of_val(&self.lod0_meshlets_count) as u64
            + std::mem::size_of_val(&self._padding1) as u64
            + std::mem::size_of_val(&self._padding2) as u64
    }
    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut GpuBuffer) {
        buffer.add_to_gpu_buffer(render_context, &[self.view]);
        buffer.add_to_gpu_buffer(render_context, &[self.mesh_flags]);
        buffer.add_to_gpu_buffer(render_context, &[self.lod0_meshlets_count]);
        buffer.add_to_gpu_buffer(render_context, &[self._padding1]);
        buffer.add_to_gpu_buffer(render_context, &[self._padding2]);
    }
}

pub struct CullingPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    commands: DrawCommandsBuffer,
    meshes: MeshesBuffer,
    meshlets: MeshletsBuffer,
    bhv: BVHBuffer,
    culling_data: CullingData,
    meshlet_culling_data: ArrayU32,
    processing_data: ArrayU32,
    listener: Listener,
    update_camera: bool,
    update_meshes: bool,
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
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let compute_data = ComputePassData {
            name: CULLING_PASS_NAME.to_string(),
            pipelines: vec![PathBuf::from(CULLING_PIPELINE)],
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
            constant_data: render_context.global_buffers().constant_data.clone(),
            commands: render_context.global_buffers().draw_commands.clone(),
            meshes: render_context.global_buffers().meshes.clone(),
            meshlets: render_context.global_buffers().meshlets.clone(),
            bhv: render_context.global_buffers().bvh.clone(),
            binding_data: BindingData::new(render_context, CULLING_PASS_NAME),
            culling_data: CullingData::default(),
            meshlet_culling_data: ArrayU32::default(),
            processing_data: ArrayU32::default(),
            listener,
            update_camera: true,
            update_meshes: true,
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

        let num_meshlets = self.meshlets.read().unwrap().item_count();
        const INITIAL_OFFSET: u32 = 2;
        let mut max_meshlets_size = INITIAL_OFFSET + num_meshlets as u32 + ATOMIC_SIZE;
        max_meshlets_size += ATOMIC_SIZE - (max_meshlets_size % ATOMIC_SIZE);
        //if self.update_meshes {
        let mut meshlet_data = vec![0; max_meshlets_size as usize];
        let mut processing_data = vec![0; max_meshlets_size as usize];
        let mut lod0_meshlets_count = INITIAL_OFFSET;
        self.meshes.read().unwrap().for_each_entry(|_, mesh| {
            let meshlets_start = mesh.meshlets_offset + (mesh.lods_meshlets_offset[0] >> 16);
            let meshlets_end = mesh.meshlets_offset + (mesh.lods_meshlets_offset[0] & 0x0000FFFF);
            for i in meshlets_start..meshlets_end {
                meshlet_data[lod0_meshlets_count as usize] = i;
                processing_data[i as usize] = 1;
                lod0_meshlets_count += 1;
            }
        });
        lod0_meshlets_count -= INITIAL_OFFSET;
        meshlet_data[0] = lod0_meshlets_count as _;
        self.culling_data.lod0_meshlets_count = lod0_meshlets_count as _;
        self.culling_data.set_dirty(true);

        self.meshlet_culling_data.write().unwrap().set(meshlet_data);
        self.processing_data.write().unwrap().set(processing_data);
        //}

        let draw_command_type = self.draw_commands_type();

        if let Some(commands) = self.commands.write().unwrap().get_mut(&mesh_flags) {
            let commands = commands.map.get_mut(&draw_command_type).unwrap();
            if commands.commands.is_empty() {
                return;
            }
            commands.counter.count = 0;
            commands.counter.set_dirty(true);

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
                    &mut *self.bhv.write().unwrap(),
                    Some("BHV"),
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
                    &mut *self.meshlet_culling_data.write().unwrap(),
                    Some("MeshletCullingData"),
                    BindingInfo {
                        group_index: 1,
                        binding_index: 2,
                        stage: ShaderStage::Compute,
                        flags: BindingFlags::ReadWrite,
                    },
                )
                .add_storage_buffer(
                    &mut *self.processing_data.write().unwrap(),
                    Some("CullingResult"),
                    BindingInfo {
                        group_index: 1,
                        binding_index: 3,
                        stage: ShaderStage::Compute,
                        flags: BindingFlags::ReadWrite | BindingFlags::Indirect,
                    },
                );

            let mut pass = self.compute_pass.get_mut();
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

        let pass = self.compute_pass.get();
        pass.dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            1,
            1,
            1,
        );
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
            })
            .process_messages(|_e: &DataTypeResourceEvent<Mesh>| {
                self.update_meshes = true;
            })
            .process_messages(|_e: &ResourceEvent<Mesh>| {
                self.update_meshes = true;
            });
    }
}
