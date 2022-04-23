use std::path::{Path, PathBuf};

use inox_log::debug_log;
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedData, SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file, SerializeFile};

use crate::{
    BindingData, BindingState, CullingModeType, DataBuffer, InstanceData, PipelineData,
    PolygonModeType, RenderContext, Shader, VertexBufferLayoutBuilder, VertexFormat,
    FRAGMENT_SHADER_ENTRY_POINT, SHADER_ENTRY_POINT, VERTEX_SHADER_ENTRY_POINT,
};

pub type PipelineId = ResourceId;

pub struct Pipeline {
    path: PathBuf,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    data: PipelineData,
    format: Option<wgpu::TextureFormat>,
    vertex_shader: Handle<Shader>,
    fragment_shader: Handle<Shader>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    binding_data: BindingData,
}

impl Clone for Pipeline {
    fn clone(&self) -> Self {
        let (vertex_shader, fragment_shader) =
            Self::load_shaders(&self.data, &self.shared_data, &self.message_hub);
        Self {
            path: self.path.clone(),
            data: self.data.clone(),
            shared_data: self.shared_data.clone(),
            message_hub: self.message_hub.clone(),
            vertex_shader: Some(vertex_shader),
            fragment_shader: Some(fragment_shader),
            format: None,
            render_pipeline: None,
            binding_data: BindingData::default(),
        }
    }
}

impl ResourceTrait for Pipeline {
    type OnCreateData = ();

    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &PipelineId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(
        &mut self,
        _shared_data: &SharedData,
        _message_hub: &MessageHubRc,
        _id: &PipelineId,
    ) {
        self.render_pipeline = None;
        self.vertex_shader = None;
        self.fragment_shader = None;
    }
    fn on_copy(&mut self, other: &Self)
    where
        Self: Sized,
    {
        *self = other.clone();
    }
}

impl SerializableResource for Pipeline {
    fn set_path(&mut self, path: &Path) -> &mut Self {
        self.path = path.to_path_buf();
        self
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
    type OnCreateData = <Self as ResourceTrait>::OnCreateData;

    fn new(_id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            path: PathBuf::new(),
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            data: PipelineData::default(),
            format: None,
            vertex_shader: None,
            fragment_shader: None,
            render_pipeline: None,
            binding_data: BindingData::default(),
        }
    }

    fn invalidate(&mut self) -> &mut Self {
        self.format = None;
        self
    }
    fn is_initialized(&self) -> bool {
        self.vertex_shader.is_some()
            && self.fragment_shader.is_some()
            && self.render_pipeline.is_some()
            && self.vertex_format() != 0
    }
    fn deserialize_data(
        path: &std::path::Path,
        registry: &SerializableRegistryRc,
        f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
        read_from_file::<Self::DataType>(path, registry, f);
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: ResourceId,
        data: Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let canonicalized_pipeline_data = data.canonicalize_paths();
        let mut pipeline = Self::new(id, shared_data, message_hub);
        pipeline.data = canonicalized_pipeline_data;
        let (vertex_shader, fragment_shader) =
            Self::load_shaders(&pipeline.data, shared_data, message_hub);
        pipeline.vertex_shader = Some(vertex_shader);
        pipeline.fragment_shader = Some(fragment_shader);
        pipeline
    }
}

impl Pipeline {
    pub fn data(&self) -> &PipelineData {
        &self.data
    }
    pub fn binding_data(&self) -> &BindingData {
        &self.binding_data
    }
    pub fn binding_data_mut(&mut self) -> &mut BindingData {
        &mut self.binding_data
    }
    pub fn vertex_format(&self) -> u32 {
        VertexFormat::to_bits(self.data.vertex_format.as_slice())
    }
    pub fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        self.render_pipeline.as_ref().unwrap()
    }
    fn load_shaders(
        data: &PipelineData,
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
    ) -> (Resource<Shader>, Resource<Shader>) {
        let vertex_shader =
            Shader::request_load(shared_data, message_hub, data.vertex_shader.as_path(), None);
        let fragment_shader = if data.vertex_shader == data.fragment_shader {
            vertex_shader.clone()
        } else {
            Shader::request_load(
                shared_data,
                message_hub,
                data.fragment_shader.as_path(),
                None,
            )
        };
        (vertex_shader, fragment_shader)
    }
    pub fn init(
        &mut self,
        context: &RenderContext,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        format: &wgpu::TextureFormat,
        constant_data_buffer: &DataBuffer,
        dynamic_data_buffer: &DataBuffer,
    ) -> bool {
        inox_profiler::scoped_profile!("pipeline::init");
        if self.vertex_shader.is_none() || self.fragment_shader.is_none() {
            return false;
        }
        if let Some(shader) = self.vertex_shader.as_ref() {
            if !shader.get().is_initialized() {
                if !shader.get_mut().init(context) {
                    return false;
                }
                self.format = None;
            }
        }
        if let Some(shader) = self.fragment_shader.as_ref() {
            if !shader.get().is_initialized() {
                if !shader.get_mut().init(context) {
                    return false;
                }
                self.format = None;
            }
        }
        let binding_state = self.binding_data.send_to_gpu(
            context,
            self.data.binding_data.as_slice(),
            constant_data_buffer,
            dynamic_data_buffer,
        );
        if binding_state == BindingState::Error {
            return false;
        }
        let is_different_format = self.format != Some(*format);
        if binding_state == BindingState::Bound && !is_different_format {
            return true;
        }
        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        self.binding_data.bind_group_layout(),
                        texture_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(
                        format!(
                            "Render Pipeline [{:?}]",
                            self.path
                                .file_stem()
                                .unwrap_or_default()
                                .to_str()
                                .unwrap_or_default()
                        )
                        .as_str(),
                    ),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: self.vertex_shader.as_ref().unwrap().get().module(),
                        entry_point: if self.data.vertex_shader == self.data.fragment_shader {
                            VERTEX_SHADER_ENTRY_POINT
                        } else {
                            SHADER_ENTRY_POINT
                        },
                        buffers: &[
                            VertexBufferLayoutBuilder::create_from_vertex_data_attribute(
                                &self.data.vertex_format,
                            )
                            .build(),
                            InstanceData::descriptor().build(),
                        ],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: self.fragment_shader.as_ref().unwrap().get().module(),
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
                        front_face: wgpu::FrontFace::default(),
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
        self.render_pipeline = Some(render_pipeline);
        true
    }

    pub fn check_shaders_to_reload(&mut self, path_as_string: String) {
        if path_as_string.contains(self.data.vertex_shader.to_str().unwrap())
            && !self.data.vertex_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            debug_log!("Vertex Shader {:?} will be reloaded", path_as_string);
        }
        if path_as_string.contains(self.data.fragment_shader.to_str().unwrap())
            && !self.data.fragment_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            debug_log!("Fragment Shader {:?} will be reloaded", path_as_string);
        }
    }
}
