use std::path::{Path, PathBuf};

use sabi_math::matrix4_to_array;
use sabi_messenger::MessengerRw;
use sabi_profiler::debug_log;
use sabi_resources::{
    BufferData, DataTypeResource, ResourceId, ResourceTrait, SerializableResource, SharedData,
    SharedDataRc,
};
use sabi_serialize::{read_from_file, SerializeFile};
use wgpu::{util::DrawIndexedIndirect, ShaderModule};

use crate::{
    create_shader, CullingModeType, GpuBuffer, InstanceData, Mesh, MeshId, PipelineData,
    PipelineIdentifier, PolygonModeType, RenderContext, VertexData, FRAGMENT_SHADER_ENTRY_POINT,
    INVALID_INDEX, SHADER_ENTRY_POINT, VERTEX_SHADER_ENTRY_POINT,
};

pub type PipelineId = ResourceId;

#[derive(Default)]
pub struct Pipeline {
    path: PathBuf,
    data: PipelineData,
    format: Option<wgpu::TextureFormat>,
    vertex_shader: Option<ShaderModule>,
    fragment_shader: Option<ShaderModule>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    instance_buffer:
        GpuBuffer<InstanceData, { wgpu::BufferUsages::bits(&wgpu::BufferUsages::VERTEX) }>,
    indirect_buffer: GpuBuffer<
        wgpu::util::DrawIndexedIndirect,
        { wgpu::BufferUsages::bits(&wgpu::BufferUsages::INDIRECT) },
    >,
}

impl Clone for Pipeline {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            data: self.data.clone(),
            format: None,
            vertex_shader: None,
            fragment_shader: None,
            render_pipeline: None,
            instance_buffer: GpuBuffer::default(),
            indirect_buffer: GpuBuffer::default(),
        }
    }
}

impl SerializableResource for Pipeline {
    fn set_path(&mut self, path: &Path) {
        self.path = path.to_path_buf();
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn extension() -> &'static str {
        PipelineData::extension()
    }
}

impl DataTypeResource for Pipeline {
    type DataType = PipelineData;
    type OnCreateData = ();

    fn invalidate(&mut self) {
        self.vertex_shader = None;
        self.fragment_shader = None;
    }
    fn is_initialized(&self) -> bool {
        self.vertex_shader.is_some() && self.fragment_shader.is_some()
    }
    fn deserialize_data(path: &Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }
    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _id: &PipelineId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(&mut self, _shared_data: &SharedData, _id: &PipelineId) {}

    fn create_from_data(
        _shared_data: &SharedDataRc,
        _global_messenger: &MessengerRw,
        _id: ResourceId,
        data: Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let canonicalized_pipeline_data = data.canonicalize_paths();
        Self {
            data: canonicalized_pipeline_data,
            ..Default::default()
        }
    }
}

impl Pipeline {
    pub fn data(&self) -> &PipelineData {
        &self.data
    }
    pub fn identifier(&self) -> PipelineIdentifier {
        PipelineIdentifier::new(&self.data.identifier)
    }
    pub fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        self.render_pipeline.as_ref().unwrap()
    }
    pub fn init(
        &mut self,
        context: &RenderContext,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        format: &wgpu::TextureFormat,
    ) {
        if self.data.vertex_shader.to_str().unwrap().is_empty()
            || self.data.fragment_shader.to_str().unwrap().is_empty()
        {
            return;
        }
        if self.vertex_shader.is_none() {
            self.vertex_shader = create_shader(context, self.data.vertex_shader.as_path());
            self.format = None;
        }
        if self.fragment_shader.is_none() {
            self.fragment_shader = create_shader(context, self.data.fragment_shader.as_path());
            self.format = None;
        }
        if let Some(f) = &self.format {
            if f == format {
                return;
            }
        }
        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts,
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: self.vertex_shader.as_ref().unwrap(),
                        entry_point: if self.data.vertex_shader == self.data.fragment_shader {
                            VERTEX_SHADER_ENTRY_POINT
                        } else {
                            SHADER_ENTRY_POINT
                        },
                        buffers: &[
                            VertexData::descriptor().build(),
                            InstanceData::descriptor().build(),
                        ],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: self.fragment_shader.as_ref().unwrap(),
                        entry_point: if self.data.vertex_shader == self.data.fragment_shader {
                            FRAGMENT_SHADER_ENTRY_POINT
                        } else {
                            SHADER_ENTRY_POINT
                        },
                        targets: &[wgpu::ColorTargetState {
                            format: *format,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: self.data.src_color_blend_factor.into(),
                                    dst_factor: self.data.dst_color_blend_factor.into(),
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent {
                                    src_factor: self.data.src_alpha_blend_factor.into(),
                                    dst_factor: self.data.dst_alpha_blend_factor.into(),
                                    operation: wgpu::BlendOperation::Add,
                                },
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        }],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: match &self.data.culling {
                            CullingModeType::Back => Some(wgpu::Face::Back),
                            CullingModeType::Front => Some(wgpu::Face::Front),
                            CullingModeType::None => None,
                        },
                        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                        polygon_mode: match &self.data.mode {
                            PolygonModeType::Fill => wgpu::PolygonMode::Fill,
                            PolygonModeType::Line => wgpu::PolygonMode::Line,
                            PolygonModeType::Point => wgpu::PolygonMode::Point,
                        },
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    // If the pipeline will be used with a multiview render pass, this
                    // indicates how many array layers the attachments will have.
                    multiview: None,
                });
        self.format = Some(*format);
        self.render_pipeline = Some(render_pipeline)
    }

    pub fn check_shaders_to_reload(&mut self, path_as_string: String) {
        if path_as_string.contains(self.data.vertex_shader.to_str().unwrap())
            && !self.data.vertex_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            debug_log(format!("Vertex Shader {:?} will be reloaded", path_as_string).as_str());
        }
        if path_as_string.contains(self.data.fragment_shader.to_str().unwrap())
            && !self.data.fragment_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            debug_log(format!("Fragment Shader {:?} will be reloaded", path_as_string).as_str());
        }
    }

    pub fn send_to_gpu(&mut self, context: &RenderContext) {
        self.instance_buffer.send_to_gpu(context);
        self.indirect_buffer.send_to_gpu(context);
    }

    pub fn instance_buffer(&self) -> Option<wgpu::BufferSlice> {
        if let Some(buffer) = self.instance_buffer.gpu_buffer() {
            return Some(buffer.slice(..));
        }
        None
    }
    pub fn instances(&self) -> &[InstanceData] {
        self.instance_buffer.data()
    }
    pub fn indirect(&self, index: usize) -> &DrawIndexedIndirect {
        &self.indirect_buffer.data()[index]
    }
    pub fn indirect_buffer(&self) -> Option<&wgpu::Buffer> {
        self.indirect_buffer.gpu_buffer()
    }

    pub fn add_mesh_to_instance_buffer(&mut self, mesh_id: &MeshId, mesh: &Mesh) {
        if self.instance_buffer.get(mesh_id).is_none() {
            let instance = InstanceData {
                id: mesh.draw_index() as _,
                matrix: matrix4_to_array(mesh.matrix()),
                draw_area: mesh.draw_area().into(),
                material_index: mesh
                    .material()
                    .as_ref()
                    .map_or(INVALID_INDEX, |m| m.get().uniform_index()),
            };
            self.instance_buffer.add(mesh_id, &[instance]);
        } else {
            let data = self.instance_buffer.data();
            let instance_index = self.instance_buffer.get(mesh_id).unwrap().start;
            let mut instance_data = data[instance_index];
            instance_data.matrix = matrix4_to_array(mesh.matrix());
            instance_data.material_index = mesh
                .material()
                .as_ref()
                .map_or(INVALID_INDEX, |m| m.get().uniform_index());
            self.instance_buffer
                .update(instance_index as _, &[instance_data]);
        }
    }
    pub fn add_mesh_to_indirect_buffer(
        &mut self,
        mesh_id: &MeshId,
        vertex_data: &BufferData,
        index_data: &BufferData,
    ) {
        if self.indirect_buffer.get(mesh_id).is_none() {
            let instance_index = self.instance_buffer.get(mesh_id).unwrap().start;
            self.indirect_buffer.add(
                mesh_id,
                &[wgpu::util::DrawIndexedIndirect {
                    vertex_count: index_data.len() as _,
                    instance_count: 1,
                    base_index: index_data.start as _,
                    vertex_offset: vertex_data.start as _,
                    base_instance: instance_index as _,
                }],
            );
        }
    }
    pub fn remove_mesh(&mut self, mesh_id: &MeshId) {
        self.instance_buffer.remove(mesh_id);
        self.indirect_buffer.remove(mesh_id);
    }
}