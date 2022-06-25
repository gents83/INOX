use std::path::PathBuf;

use crate::{
    AsBinding, BindingData, ComputePass, ComputePassData, GpuBuffer, Pass, RenderContext,
    RenderCoreContext, RenderPipeline, DEFAULT_PIPELINE,
};

use inox_core::ContextRc;
use inox_math::{normalize_plane, Matrix, Matrix4, Vector3};
use inox_resources::{DataTypeResource, Resource, SerializableResource, SharedDataRc};
use inox_uid::generate_random_uid;

pub const CULLING_PIPELINE: &str = "pipelines/Culling.compute_pipeline";
pub const CULLING_PASS_NAME: &str = "CullingPass";

pub enum CullingFlags {
    None = 0,
    Freezed = (1 << 1) as _,
}

#[derive(Default)]
struct MeshData {
    _position: [f32; 3],
    _scale: f32,
    _orientation: [f32; 4],
}

#[derive(Default)]
struct Meshes {
    meshes: Vec<MeshData>,
}

impl AsBinding for Meshes {
    fn is_dirty(&self) -> bool {
        true
    }
    fn set_dirty(&mut self, _is_dirty: bool) {}
    fn size(&self) -> u64 {
        (std::mem::size_of::<MeshData>() * self.meshes.len()) as _
    }

    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut GpuBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, self.meshes.as_slice());
    }
}

#[derive(Default)]
struct CullingPassData {
    is_dirty: bool,
    cam_pos: [f32; 3],
    _padding: f32,
    frustum: [[f32; 4]; 4],
}

impl AsBinding for CullingPassData {
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    fn set_dirty(&mut self, is_dirty: bool) {
        self.is_dirty = is_dirty;
    }
    fn size(&self) -> u64 {
        std::mem::size_of_val(&self.cam_pos) as u64
            + std::mem::size_of_val(&self._padding) as u64
            + std::mem::size_of_val(&self.frustum) as u64
    }

    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut GpuBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, &[self.cam_pos]);
        buffer.add_to_gpu_buffer(render_core_context, &[self._padding]);
        buffer.add_to_gpu_buffer(render_core_context, &[self.frustum]);
    }
}

pub struct CullingPass {
    compute_pass: Resource<ComputePass>,
    _default_pipeline: Resource<RenderPipeline>,
    data: CullingPassData,
    binding_data: BindingData,
    _shared_data: SharedDataRc,
    num_meshlets: usize,
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
            _default_pipeline: RenderPipeline::request_load(
                context.shared_data(),
                context.message_hub(),
                PathBuf::from(DEFAULT_PIPELINE).as_path(),
                None,
            ),
            data: CullingPassData {
                is_dirty: true,
                ..Default::default()
            },
            binding_data: BindingData::default(),
            _shared_data: context.shared_data().clone(),
            num_meshlets: 0,
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        /*
        let pipeline_id = self.default_pipeline.id();
        let vertex_format = self.default_pipeline.get().vertex_format();
        let mut meshes = Meshes::default();

        if let Some(mesh_ids) = render_context.graphics_data.get().meshes(pipeline_id) {
            mesh_ids.iter().for_each(|mesh_id| {
                if let Some(mesh) = self.shared_data.get_resource::<Mesh>(mesh_id) {
                    let (t, r, s) = mesh.get().matrix().get_translation_rotation_scale();
                    let q = Quaternion::from_euler_angles(r);
                    meshes.meshes.push(MeshData {
                        _position: t.into(),
                        _scale: s.x,
                        _orientation: q.into(),
                    });
                }
            });
        }

        let commands = render_context
            .graphics_data
            .get()
            .create_commands(pipeline_id);

        if commands.is_none() {
            return;
        }

        if let Some(meshlets) = render_context
            .graphics_data
            .get_mut()
            .get_meshlets(&vertex_format)
        {
            self.num_meshlets = meshlets.data.len();

            if self.num_meshlets == 0 || meshes.meshes.is_empty() {
                return;
            }
            let mut commands = Commands {
                commands: commands.unwrap(),
            };
            {
                let mut binding_buffers =
                    render_context.binding_data_buffer.buffers.write().unwrap();
                binding_buffers.remove(pipeline_id);
            }
            let usage = wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::MAP_READ;
            render_context.binding_data_buffer.bind_buffer_with_id(
                pipeline_id,
                &mut commands,
                usage,
                &render_context.core,
            );
            self.binding_data.mark_as_dirty();

            self.binding_data
                .add_uniform_data(
                    &render_context.core,
                    &render_context.binding_data_buffer,
                    &mut self.data,
                    BindingInfo {
                        group_index: 0,
                        binding_index: 0,
                        stage: ShaderStage::Compute,
                        ..Default::default()
                    },
                )
                .add_storage_data(
                    &render_context.core,
                    &render_context.binding_data_buffer,
                    meshlets,
                    BindingInfo {
                        group_index: 0,
                        binding_index: 1,
                        stage: ShaderStage::Compute,
                        read_only: true,
                        ..Default::default()
                    },
                )
                .add_storage_data(
                    &render_context.core,
                    &render_context.binding_data_buffer,
                    &mut meshes,
                    BindingInfo {
                        group_index: 0,
                        binding_index: 2,
                        stage: ShaderStage::Compute,
                        read_only: true,
                        ..Default::default()
                    },
                )
                .bind_storage_buffer(
                    pipeline_id,
                    BindingInfo {
                        group_index: 0,
                        binding_index: 3,
                        stage: ShaderStage::Compute,
                        read_only: false,
                        ..Default::default()
                    },
                )
                .send_to_gpu(render_context);
        }

        if self.num_meshlets == 0 || meshes.meshes.is_empty() {
            return;
        }
        */
        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &self.binding_data);
    }

    fn update(&mut self, render_context: &mut RenderContext) {
        if self.num_meshlets == 0 {
            return;
        }

        let pass = self.compute_pass.get();

        let mut encoder = render_context.core.new_encoder();
        let compute_pass = pass.begin(&self.binding_data, &mut encoder);
        let num_meshlet_per_group = 32;
        let count = (self.num_meshlets as u32 + num_meshlet_per_group - 1) / num_meshlet_per_group;
        pass.dispatch(compute_pass, count, 1, 1);

        render_context.core.submit(encoder);
    }
}

impl CullingPass {
    pub fn set_camera_data(&mut self, cam_pos: Vector3, view_proj: Matrix4) {
        self.data.cam_pos = cam_pos.into();
        self.data.frustum = [
            normalize_plane(view_proj.row(3) + view_proj.row(0)).into(),
            normalize_plane(view_proj.row(3) - view_proj.row(0)).into(),
            normalize_plane(view_proj.row(3) + view_proj.row(1)).into(),
            normalize_plane(view_proj.row(3) - view_proj.row(1)).into(),
        ];
        self.data.is_dirty = true;
    }
}
