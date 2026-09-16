use std::path::{Path, PathBuf};

use inox_log::debug_log;
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedDataRc,
};
use inox_serialize::{read_from_file, SerializationType, SerializeFile};

use crate::{
    BindingData, BlendFactor, ReloadError, ReloadState, RenderContext, RenderPipelineData, Shader,
    TextureFormat, VertexBufferLayoutBuilder, FRAGMENT_SHADER_ENTRY_POINT, SHADER_ENTRY_POINT,
    VERTEX_SHADER_ENTRY_POINT,
};

pub type RenderPipelineId = ResourceId;

pub struct RenderPipeline {
    path: PathBuf,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    data: RenderPipelineData,
    formats: Vec<TextureFormat>,
    vertex_shader: Handle<Shader>,
    fragment_shader: Handle<Shader>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    reload_pending: bool,
    last_reload_error: Option<ReloadError>,
    reload_state: ReloadState,
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
            formats: Vec::new(),
            render_pipeline: None,
            reload_pending: self.reload_pending,
            last_reload_error: self.last_reload_error.clone(),
            reload_state: self.reload_state.clone(),
        }
    }
}

impl ResourceTrait for RenderPipeline {
    fn invalidate(&mut self) -> &mut Self {
        self.formats = Vec::new();
        self.reload_pending = false;
        self
    }

    fn replace_from(&mut self, other: &Self) {
        self.path = other.path.clone();
        self.data = other.data.clone();
        self.vertex_shader = other.vertex_shader.clone();
        self.fragment_shader = other.fragment_shader.clone();
        self.reload_pending = true;
        self.last_reload_error = None;
        self.reload_state.stage();
    }

    fn is_initialized(&self) -> bool {
        self.vertex_shader.is_some()
            && self.fragment_shader.is_some()
            && self.render_pipeline.is_some()
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

    fn deserialize_data(path: &std::path::Path, f: Box<dyn FnMut(Self::DataType) + 'static>) {
        read_from_file::<Self::DataType>(path, SerializationType::Json, f);
    }
}

impl DataTypeResource for RenderPipeline {
    type DataType = RenderPipelineData;

    fn new(_id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            path: PathBuf::new(),
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            data: RenderPipelineData::default(),
            formats: Vec::new(),
            vertex_shader: None,
            fragment_shader: None,
            render_pipeline: None,
            reload_pending: false,
            last_reload_error: None,
            reload_state: ReloadState::default(),
        }
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
        render_formats: Vec<&TextureFormat>,
        depth_format: Option<&TextureFormat>,
        binding_data: &BindingData,
        vertex_layout: Option<VertexBufferLayoutBuilder>,
        instance_layout: Option<VertexBufferLayoutBuilder>,
    ) -> bool {
        inox_profiler::scoped_profile!("render_pipeline::init");
        if self.vertex_shader.is_none() || self.fragment_shader.is_none() {
            return false;
        }
        if let Some(shader) = self.vertex_shader.as_ref() {
            if !shader.get().is_initialized() || shader.get().has_pending_reload() {
                if let Err(error) = shader.get_mut().init(context) {
                    self.last_reload_error = Some(error.clone());
                    self.reload_state.fail(error);
                    return false;
                }
            }
        }
        if let Some(shader) = self.fragment_shader.as_ref() {
            if !shader.get().is_initialized() || shader.get().has_pending_reload() {
                if let Err(error) = shader.get_mut().init(context) {
                    self.last_reload_error = Some(error.clone());
                    self.reload_state.fail(error);
                    return false;
                }
            }
        }
        let is_same_format = if render_formats.is_empty() {
            !self.formats.is_empty()
                && self.formats[0] == context.webgpu.config.read().unwrap().format.into()
        } else {
            let count = self
                .formats
                .iter()
                .zip(&render_formats)
                .filter(|&(a, &b)| a == b)
                .count();
            count == self.formats.len() && count == render_formats.len()
        };
        if is_same_format && !self.reload_pending {
            return true;
        }
        let pipeline_render_formats = if render_formats.is_empty() {
            if self.data.sampling_count > 1 {
                return false;
            }
            vec![context.webgpu.config.read().unwrap().format]
        } else {
            render_formats
                .iter()
                .map(|&f| {
                    let format: wgpu::TextureFormat = (*f).into();
                    format
                })
                .collect()
        };
        let render_pipeline_layout =
            context
                .webgpu
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: binding_data
                        .bind_group_layouts()
                        .iter()
                        .map(Some)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    ..Default::default()
                });

        let mut vertex_state_buffers = Vec::new();
        if let Some(vertex_layout) = vertex_layout.as_ref() {
            vertex_state_buffers.push(Some(vertex_layout.build()));
        }
        if let Some(instance_layout) = instance_layout.as_ref() {
            vertex_state_buffers.push(Some(instance_layout.build()));
        }

        let render_pipeline = {
            inox_profiler::scoped_profile!("render_pipeline::create[{}]", self.name());
            context.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                cache: None,
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: self.vertex_shader.as_ref().unwrap().get().module(),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    entry_point: if self.data.vertex_shader == self.data.fragment_shader {
                        Some(VERTEX_SHADER_ENTRY_POINT)
                    } else {
                        Some(SHADER_ENTRY_POINT)
                    },
                    buffers: vertex_state_buffers.as_slice(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: self.fragment_shader.as_ref().unwrap().get().module(),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    entry_point: if self.data.vertex_shader == self.data.fragment_shader {
                        Some(FRAGMENT_SHADER_ENTRY_POINT)
                    } else {
                        Some(SHADER_ENTRY_POINT)
                    },
                    targets: pipeline_render_formats
                        .iter()
                        .map(|&render_format| {
                            Some(wgpu::ColorTargetState {
                                format: render_format,
                                blend: if self.data.src_color_blend_factor == BlendFactor::Zero
                                    && self.data.dst_color_blend_factor == BlendFactor::Zero
                                {
                                    None
                                } else {
                                    Some(wgpu::BlendState {
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
                                    })
                                },
                                write_mask: wgpu::ColorWrites::ALL,
                            })
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
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
                    format: (*format).into(),
                    depth_write_enabled: Some(self.data.depth_write_enabled),
                    depth_compare: Some(self.data.depth_compare.into()),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: self.data.sampling_count,
                    ..Default::default()
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview_mask: None,
            })
        };
        match render_pipeline {
            Ok(render_pipeline) => {
                self.formats = pipeline_render_formats.iter().map(|&f| f.into()).collect();
                self.render_pipeline = Some(render_pipeline);
                self.reload_pending = false;
                self.last_reload_error = None;
                self.reload_state.commit();
                true
            }
            Err(diagnostic) => {
                let error = ReloadError::new(
                    &self.path,
                    "render pipeline",
                    diagnostic,
                    self.render_pipeline.is_some(),
                );
                debug_log!(
                    "Render pipeline reload failed for {:?}: {}",
                    error.path,
                    error.diagnostic
                );
                self.last_reload_error = Some(error.clone());
                self.reload_state.fail(error);
                false
            }
        }
    }

    pub fn check_shaders_to_reload(&mut self, path_as_string: String) {
        let vertex_shader_path = self.data.vertex_shader.to_string_lossy();
        if !vertex_shader_path.is_empty() && path_as_string.contains(vertex_shader_path.as_ref()) {
            self.reload_pending = true;
            debug_log!("Vertex Shader {:?} will be reloaded", path_as_string);
        }
        let fragment_shader_path = self.data.fragment_shader.to_string_lossy();
        if !fragment_shader_path.is_empty()
            && path_as_string.contains(fragment_shader_path.as_ref())
        {
            self.reload_pending = true;
            debug_log!("Fragment Shader {:?} will be reloaded", path_as_string);
        }
    }

    pub fn reload_state(&self) -> &ReloadState {
        &self.reload_state
    }

    pub fn reload_state_mut(&mut self) -> &mut ReloadState {
        &mut self.reload_state
    }

    pub fn record_reload_failure(&mut self, error: ReloadError) {
        self.last_reload_error = Some(error.clone());
        self.reload_state.fail(error);
    }

    pub fn last_reload_error(&self) -> Option<&ReloadError> {
        self.last_reload_error.as_ref()
    }
}
