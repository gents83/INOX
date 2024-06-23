use std::path::PathBuf;

use inox_bvh::GPUBVHNode;
use inox_render::{
    AsBinding, BindingData, BindingFlags, BindingInfo, BufferRef, CommandBuffer, ComputePass,
    ComputePassData, ConstantDataRw, DrawIndexedCommand, GPUBuffer, GPUInstance, GPUMeshlet,
    GPUTransform, GPUVector, MeshFlags, Pass, RenderContext, RenderContextRc, ShaderStage, Texture,
    TextureView,
};

use inox_commands::CommandParser;
use inox_core::ContextRc;

use inox_messenger::{implement_message, Listener};
use inox_resources::{DataTypeResource, Handle, Resource};
use inox_uid::generate_random_uid;

use crate::{
    CommandsData, ACTIVE_INSTANCE_DATA_ID, COMMANDS_DATA_ID, INSTANCE_DATA_ID, MESHLETS_COUNT_ID,
};

pub const CULLING_PIPELINE: &str = "pipelines/ComputeCulling.compute_pipeline";
pub const CULLING_PASS_NAME: &str = "CullingPass";

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct MeshletLodLevel(pub u32);

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
    view: [[f32; 4]; 4],
    mesh_flags: u32,
    lod0_meshlets_count: u32,
    _padding1: u32,
    _padding2: u32,
}

impl AsBinding for CullingData {
    fn count(&self) -> usize {
        1
    }
    fn size(&self) -> u64 {
        std::mem::size_of_val(&self.view) as u64
            + std::mem::size_of_val(&self.mesh_flags) as u64
            + std::mem::size_of_val(&self.lod0_meshlets_count) as u64
            + std::mem::size_of_val(&self._padding1) as u64
            + std::mem::size_of_val(&self._padding2) as u64
    }
    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut BufferRef) {
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
    bvh: GPUBuffer<GPUBVHNode>,
    meshlets: GPUBuffer<GPUMeshlet>,
    transforms: GPUVector<GPUTransform>,
    commands: GPUVector<DrawIndexedCommand>,
    instances: GPUVector<GPUInstance>,
    active_instances: GPUVector<GPUInstance>,
    commands_data: GPUVector<CommandsData>,
    meshlets_count: GPUVector<u32>,
    hzb_texture: Handle<Texture>,
    culling_data: CullingData,
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
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
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
            bvh: render_context.global_buffers().buffer::<GPUBVHNode>(),
            meshlets: render_context.global_buffers().buffer::<GPUMeshlet>(),
            transforms: render_context.global_buffers().vector::<GPUTransform>(),
            commands: render_context
                .global_buffers()
                .vector::<DrawIndexedCommand>(),
            instances: render_context
                .global_buffers()
                .vector_with_id::<GPUInstance>(INSTANCE_DATA_ID),
            active_instances: render_context
                .global_buffers()
                .vector_with_id::<GPUInstance>(ACTIVE_INSTANCE_DATA_ID),
            commands_data: render_context
                .global_buffers()
                .vector_with_id::<CommandsData>(COMMANDS_DATA_ID),
            meshlets_count: render_context
                .global_buffers()
                .vector_with_id::<u32>(MESHLETS_COUNT_ID),
            hzb_texture: None,
            binding_data: BindingData::new(render_context, CULLING_PASS_NAME),
            culling_data: CullingData::default(),
            listener,
            update_camera: true,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("compute_culling_pass::init");

        self.process_messages();

        let commands_count = self.commands.read().unwrap().len();
        if self.instances.read().unwrap().is_empty() || commands_count == 0 {
            return;
        }

        let mesh_flags = MeshFlags::Visible | MeshFlags::Opaque;

        let flags: u32 = mesh_flags.into();
        if self.culling_data.mesh_flags != flags {
            self.culling_data.mesh_flags = flags;
            self.culling_data.mark_as_dirty(render_context);
        }

        if self.update_camera {
            let view = self.constant_data.read().unwrap().view();
            if self.culling_data.view != view {
                self.culling_data.view = view;
                self.culling_data.mark_as_dirty(render_context);
            }
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
                &mut *self.bvh.write().unwrap(),
                Some("BVH"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.transforms.write().unwrap(),
                Some("Transforms"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.instances.write().unwrap(),
                Some("Instances"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.commands_data.write().unwrap(),
                Some("CommandsData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 6,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.commands.write().unwrap(),
                Some("Commands"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 7,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite,
                    count: Some(commands_count),
                },
            )
            .add_storage_buffer(
                &mut *self.active_instances.write().unwrap(),
                Some("ActiveInstances"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshlets_count.write().unwrap(),
                Some("MeshletsCount"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_texture(
                self.hzb_texture.as_ref().unwrap().id(),
                0,
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_default_sampler(
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
                inox_render::SamplerType::Unfiltered,
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
        inox_profiler::scoped_profile!("compute_culling_pass::update");

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

impl CullingPass {
    pub fn set_hzb_texture(&mut self, hzb_texture: &Resource<Texture>) {
        self.hzb_texture = Some(hzb_texture.clone());
    }
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
