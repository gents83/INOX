use std::path::PathBuf;

use crate::{
    AsBufferBinding, BindingData, BindingInfo, ComputePass, ComputePassData, DataBuffer, Mesh,
    Pass, RenderContext, RenderCoreContext, RenderPipeline, ShaderStage, DEFAULT_PIPELINE,
};

use inox_core::ContextRc;
use inox_math::{compute_frustum, Faces, Mat4Ops, Matrix4, Quat, Quaternion};
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

impl AsBufferBinding for Meshes {
    fn is_dirty(&self) -> bool {
        true
    }
    fn set_dirty(&mut self, _is_dirty: bool) {}
    fn size(&self) -> u64 {
        (std::mem::size_of::<MeshData>() * self.meshes.len()) as _
    }

    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut DataBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, self.meshes.as_slice());
    }
}

#[derive(Default)]
struct CullingPassData {
    is_dirty: bool,
    cam_pos: [f32; 3],
    flags: u32,
    frustum: [[f32; 4]; Faces::Count as usize],
}

impl AsBufferBinding for CullingPassData {
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    fn set_dirty(&mut self, is_dirty: bool) {
        self.is_dirty = is_dirty;
    }
    fn size(&self) -> u64 {
        std::mem::size_of::<Self>() as _
    }

    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut DataBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, &[self]);
    }
}

pub struct CullingPass {
    compute_pass: Resource<ComputePass>,
    default_pipeline: Resource<RenderPipeline>,
    data: CullingPassData,
    binding_data: BindingData,
    shared_data: SharedDataRc,
    num_meshlets: usize,
}
unsafe impl Send for CullingPass {}
unsafe impl Sync for CullingPass {}

impl Pass for CullingPass {
    fn name(&self) -> &str {
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
            ),
            default_pipeline: RenderPipeline::request_load(
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
            shared_data: context.shared_data().clone(),
            num_meshlets: 0,
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
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

        if let Some(meshlets) = render_context
            .graphics_data
            .get_mut()
            .get_meshlets(&vertex_format)
        {
            self.num_meshlets = meshlets.data.len();

            if self.num_meshlets == 0 || meshes.meshes.is_empty() {
                return;
            }
            if !render_context.binding_data_buffer.has_buffer(pipeline_id) {
                return;
            }

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
                    },
                )
                .bind_storage_buffer(
                    pipeline_id,
                    BindingInfo {
                        group_index: 0,
                        binding_index: 3,
                        stage: ShaderStage::Compute,
                        read_only: false,
                    },
                )
                .send_to_gpu(render_context);
        }

        if self.num_meshlets == 0 || meshes.meshes.is_empty() {
            return;
        }

        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &self.binding_data);
    }

    fn update(&mut self, render_context: &RenderContext) {
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
    pub fn set_camera_data(&mut self, view: &Matrix4, proj: &Matrix4) {
        let frustum = compute_frustum(view, proj);
        for i in 0..Faces::Count as usize {
            self.data.frustum[i] = frustum.faces[i].into();
        }
        self.data.cam_pos = view.translation().into();
        self.data.is_dirty = true;
    }

    pub fn set_flags(&mut self, flags: CullingFlags) {
        self.data.flags = flags as _;
        self.data.is_dirty = true;
    }
}
