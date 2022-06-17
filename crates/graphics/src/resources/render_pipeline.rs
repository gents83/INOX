use std::path::{Path, PathBuf};

use inox_log::debug_log;
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedData, SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file, SerializeFile};

use crate::{
    BindingData, DrawInstance, DrawVertex, RenderContext, RenderPipelineData, Shader,
    FRAGMENT_SHADER_ENTRY_POINT, SHADER_ENTRY_POINT, VERTEX_SHADER_ENTRY_POINT,
};

pub type RenderPipelineId = ResourceId;

pub struct RenderPipeline {
    path: PathBuf,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    data: RenderPipelineData,
    format: Option<wgpu::TextureFormat>,
    vertex_shader: Handle<Shader>,
    fragment_shader: Handle<Shader>,
    render_pipeline: Option<wgpu::RenderPipeline>,
}

impl Clone for RenderPipeline {
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
        }
    }
}

impl ResourceTrait for RenderPipeline {
    type OnCreateData = ();

    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &RenderPipelineId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(
        &mut self,
        _shared_data: &SharedData,
        _message_hub: &MessageHubRc,
        _id: &RenderPipelineId,
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
        self.invalidate();
    }
}

impl SerializableResource for RenderPipeline {
    fn set_path(&mut self, path: &Path) -> &mut Self {
        self.path = path.to_path_buf();
        self
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn extension() -> &'static str {
        RenderPipelineData::extension()
    }
}

impl DataTypeResource for RenderPipeline {
    type DataType = RenderPipelineData;
    type OnCreateData = <Self as ResourceTrait>::OnCreateData;

    fn new(_id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            path: PathBuf::new(),
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            data: RenderPipelineData::default(),
            format: None,
            vertex_shader: None,
            fragment_shader: None,
            render_pipeline: None,
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
        data: &Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let data = data.canonicalize_paths();
        let mut pipeline = Self::new(id, shared_data, message_hub);
        pipeline.data = data;
        let (vertex_shader, fragment_shader) =
            Self::load_shaders(&pipeline.data, shared_data, message_hub);
        pipeline.vertex_shader = Some(vertex_shader);
        pipeline.fragment_shader = Some(fragment_shader);
        pipeline
    }
}

impl RenderPipeline {
    pub fn data(&self) -> &RenderPipelineData {
        &self.data
    }
    pub fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        self.render_pipeline.as_ref().unwrap()
    }
    fn load_shaders(
        data: &RenderPipelineData,
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
        render_format: &wgpu::TextureFormat,
        depth_format: Option<&wgpu::TextureFormat>,
        binding_data: &BindingData,
    ) -> bool {
        inox_profiler::scoped_profile!("render_pipeline::init");
        if self.vertex_shader.is_none() || self.fragment_shader.is_none() {
            return false;
        }
        if let Some(shader) = self.vertex_shader.as_ref() {
            if !shader.get().is_initialized() {
                if !shader.get_mut().init(context) {
                    return false;
                }
                self.render_pipeline = None;
                self.format = None;
            }
        }
        if let Some(shader) = self.fragment_shader.as_ref() {
            if !shader.get().is_initialized() {
                if !shader.get_mut().init(context) {
                    return false;
                }
                self.render_pipeline = None;
                self.format = None;
            }
        }
        let is_same_format = self.format == Some(*render_format);
        if is_same_format {
            return true;
        }
        let render_pipeline_layout =
            context
                .core
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: binding_data
                        .bind_group_layouts()
                        .iter()
                        .collect::<Vec<_>>()
                        .as_slice(),
                    push_constant_ranges: &[],
                });

        let vertex_layout = DrawVertex::descriptor(0);
        let instance_layout = DrawInstance::descriptor(vertex_layout.location());

        let render_pipeline = {
            inox_profiler::scoped_profile!("render_pipeline::crate[{}]", self.name());
            context
                .core
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
                        buffers: &[vertex_layout.build(), instance_layout.build()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: self.fragment_shader.as_ref().unwrap().get().module(),
                        entry_point: if self.data.vertex_shader == self.data.fragment_shader {
                            FRAGMENT_SHADER_ENTRY_POINT
                        } else {
                            SHADER_ENTRY_POINT
                        },
                        targets: &[wgpu::ColorTargetState {
                            format: *render_format,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: self.data.src_color_blend_factor.into(),
                                    dst_factor: self.data.dst_color_blend_factor.into(),
                                    operation: self.data.color_blend_operation.into(),
                                },
                                alpha: wgpu::BlendComponent {
                                    src_factor: self.data.src_alpha_blend_factor.into(),
                                    dst_factor: self.data.dst_alpha_blend_factor.into(),
                                    operation: self.data.alpha_blend_operation.into(),
                                },
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        }],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: self.data.front_face.into(),
                        cull_mode: self.data.culling.into(),
                        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                        polygon_mode: self.data.mode.into(),
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
                        format: *format,
                        depth_write_enabled: self.data.depth_write_enabled,
                        depth_compare: self.data.depth_compare.into(),
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    // If the pipeline will be used with a multiview render pass, this
                    // indicates how many array layers the attachments will have.
                    multiview: None,
                })
        };
        self.format = Some(*render_format);
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
