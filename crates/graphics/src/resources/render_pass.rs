use std::path::PathBuf;

use image::DynamicImage;
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedData, SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file};
use inox_uid::generate_random_uid;

use crate::{
    platform::is_indirect_mode_enabled, DataBuffer, GraphicsData, LoadOperation, Pipeline,
    RenderContext, RenderMode, RenderPassData, RenderTarget, StoreOperation, Texture,
};

pub type RenderPassId = ResourceId;

#[derive(Clone)]
pub struct RenderPass {
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    data: RenderPassData,
    pipelines: Vec<Resource<Pipeline>>,
    render_texture: Handle<Texture>,
    depth_texture: Handle<Texture>,
    is_initialized: bool,
}

pub struct RenderPassPrepareContext<'a> {
    pub context: &'a RenderContext,
    pub graphics_data: &'a mut GraphicsData,
    pub render_format: &'a wgpu::TextureFormat,
    pub depth_format: Option<&'a wgpu::TextureFormat>,
    pub texture_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub constant_data_buffer: &'a DataBuffer,
    pub dynamic_data_buffer: &'a DataBuffer,
}
pub struct RenderPassDrawContext<'a> {
    pub context: &'a RenderContext,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub texture_view: &'a wgpu::TextureView,
    pub render_format: &'a wgpu::TextureFormat,
    pub depth_view: Option<&'a wgpu::TextureView>,
    pub depth_format: Option<&'a wgpu::TextureFormat>,
    pub graphics_data: &'a Resource<GraphicsData>,
    pub texture_bind_group: &'a wgpu::BindGroup,
}

impl ResourceTrait for RenderPass {
    type OnCreateData = ();

    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &RenderPassId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(
        &mut self,
        _shared_data: &SharedData,
        _message_hub: &MessageHubRc,
        _id: &RenderPassId,
    ) {
    }
    fn on_copy(&mut self, other: &Self)
    where
        Self: Sized,
    {
        *self = other.clone();
    }
}

impl DataTypeResource for RenderPass {
    type DataType = RenderPassData;
    type OnCreateData = <Self as ResourceTrait>::OnCreateData;

    fn new(_id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            data: RenderPassData::default(),
            pipelines: Vec::new(),
            render_texture: None,
            depth_texture: None,
            is_initialized: false,
        }
    }
    fn invalidate(&mut self) -> &mut Self {
        self.is_initialized = false;
        self
    }
    fn is_initialized(&self) -> bool {
        self.is_initialized
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
        _id: ResourceId,
        data: Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let render_target = data.render_target;
        let depth_target = data.depth_target;
        let pipelines = data.pipelines.clone();
        let mut pass = Self {
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            data,
            pipelines: Vec::new(),
            render_texture: None,
            depth_texture: None,
            is_initialized: false,
        };
        pass.render_target(render_target)
            .depth_target(depth_target)
            .pipelines(pipelines);
        pass
    }
}

impl RenderPass {
    pub fn data(&self) -> &RenderPassData {
        &self.data
    }
    pub fn pipeline(&self, index: usize) -> Option<&Resource<Pipeline>> {
        self.pipelines.get(index)
    }
    pub fn pipelines(&mut self, pipelines: Vec<PathBuf>) -> &mut Self {
        self.pipelines.clear();
        pipelines.iter().for_each(|path| {
            if !path.as_os_str().is_empty() {
                let pipeline = Pipeline::request_load(
                    &self.shared_data,
                    &self.message_hub,
                    path.as_path(),
                    None,
                );
                self.pipelines.push(pipeline);
            };
        });
        self
    }
    pub fn render_target(&mut self, render_target: RenderTarget) -> &mut Self {
        self.data.render_target = render_target;
        self.render_texture = match render_target {
            RenderTarget::Texture {
                width,
                height,
                read_back: _,
            } => {
                let texture_id = generate_random_uid();
                let image = DynamicImage::new_rgba8(width, height);
                let image_data = image.to_rgba8();
                let mut texture = Texture::create_from_data(
                    &self.shared_data,
                    &self.message_hub,
                    texture_id,
                    image_data,
                );
                texture.on_create(&self.shared_data, &self.message_hub, &texture_id, None);
                let texture = self
                    .shared_data
                    .add_resource(&self.message_hub, texture_id, texture);
                Some(texture)
            }
            _ => None,
        };

        self
    }
    pub fn render_target_from_texture(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.data.render_target = RenderTarget::Texture {
            width: texture.get().width(),
            height: texture.get().height(),
            read_back: false,
        };
        self.render_texture = Some(texture.clone());
        self
    }
    pub fn depth_target(&mut self, render_target: RenderTarget) -> &mut Self {
        self.data.depth_target = render_target;
        self.depth_texture = match render_target {
            RenderTarget::Texture {
                width,
                height,
                read_back: _,
            } => {
                let texture_id = generate_random_uid();
                let image = DynamicImage::new_rgba8(width, height);
                let image_data = image.to_rgba8();
                let mut texture = Texture::create_from_data(
                    &self.shared_data,
                    &self.message_hub,
                    texture_id,
                    image_data,
                );
                texture.on_create(&self.shared_data, &self.message_hub, &texture_id, None);
                let texture = self
                    .shared_data
                    .add_resource(&self.message_hub, texture_id, texture);
                Some(texture)
            }
            _ => None,
        };
        self
    }
    pub fn depth_target_from_texture(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.data.depth_target = RenderTarget::Texture {
            width: texture.get().width(),
            height: texture.get().height(),
            read_back: false,
        };
        self.depth_texture = Some(texture.clone());
        self
    }
    pub fn render_texture(&self) -> &Handle<Texture> {
        &self.render_texture
    }
    pub fn depth_texture(&self) -> &Handle<Texture> {
        &self.depth_texture
    }
    pub fn init(&mut self, context: &mut RenderContext) {
        if self.is_initialized {
            return;
        }
        if let Some(texture) = &self.render_texture {
            let texture_handler = &mut context.texture_handler;
            if texture_handler.get_texture_atlas(texture.id()).is_none() {
                texture_handler.add_custom_texture(
                    &context.device,
                    texture.id(),
                    texture.get().width(),
                    texture.get().height(),
                    wgpu::TextureFormat::Rgba8Unorm,
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                );
            }
        }
        if let Some(texture) = &self.depth_texture {
            let texture_handler = &mut context.texture_handler;
            if texture_handler.get_texture_atlas(texture.id()).is_none() {
                texture_handler.add_custom_texture(
                    &context.device,
                    texture.id(),
                    texture.get().width(),
                    texture.get().height(),
                    wgpu::TextureFormat::Depth32Float,
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                );
            }
        } else {
            self.is_initialized = true;
        }
    }
    fn color_operations(&self, c: wgpu::Color) -> wgpu::Operations<wgpu::Color> {
        wgpu::Operations {
            load: match &self.data.load_color {
                LoadOperation::Load => wgpu::LoadOp::Load,
                _ => wgpu::LoadOp::Clear(c),
            },
            store: matches!(&self.data.store_color, StoreOperation::Store),
        }
    }

    fn depth_operations(&self) -> wgpu::Operations<f32> {
        wgpu::Operations {
            load: match &self.data.load_depth {
                LoadOperation::Load => wgpu::LoadOp::Load,
                _ => wgpu::LoadOp::Clear(1.),
            },
            store: matches!(&self.data.store_depth, StoreOperation::Store),
        }
    }

    pub fn prepare(&mut self, render_pass_prepare_context: RenderPassPrepareContext) {
        inox_profiler::scoped_profile!("render_pass::prepare");

        if render_pass_prepare_context
            .graphics_data
            .total_vertex_count()
            == 0
        {
            return;
        }
        self.pipelines.iter_mut().for_each(|pipeline| {
            let pipeline_id = pipeline.id();
            let instance_count = render_pass_prepare_context
                .graphics_data
                .instance_count(pipeline_id);
            if instance_count > 0 {
                if !pipeline.get_mut().init(
                    render_pass_prepare_context.context,
                    render_pass_prepare_context.texture_bind_group_layout,
                    render_pass_prepare_context.render_format,
                    render_pass_prepare_context.depth_format,
                    render_pass_prepare_context.constant_data_buffer,
                    render_pass_prepare_context.dynamic_data_buffer,
                ) {
                    return;
                }
                if is_indirect_mode_enabled() && self.data.render_mode == RenderMode::Indirect {
                    render_pass_prepare_context
                        .graphics_data
                        .fill_command_buffer(render_pass_prepare_context.context, pipeline_id);
                }
            }
        });
    }

    pub fn draw(&self, render_pass_context: RenderPassDrawContext) {
        inox_profiler::scoped_profile!("render_pass[{}]::draw", self.data.name);

        let graphics_data = render_pass_context.graphics_data.get();
        if graphics_data.total_vertex_count() == 0 || graphics_data.total_index_count() == 0 {
            return;
        }
        let pipelines = self.pipelines.iter().map(|h| h.get()).collect::<Vec<_>>();
        let depth_write_enabled = pipelines
            .iter()
            .any(|pipeline| pipeline.data().depth_write_enabled);
        let pipelines_id = self.pipelines.iter().map(|h| h.id()).collect::<Vec<_>>();
        let color_operations = self.color_operations(wgpu::Color::BLACK);
        let label = format!("RenderPass {}", self.data.name);
        let mut render_pass =
            render_pass_context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(label.as_str()),
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: render_pass_context.texture_view,
                        resolve_target: None,
                        ops: color_operations,
                    }],
                    depth_stencil_attachment: render_pass_context.depth_view.map(|depth_view| {
                        wgpu::RenderPassDepthStencilAttachment {
                            view: depth_view,
                            depth_ops: if depth_write_enabled {
                                let depth_operations = self.depth_operations();
                                Some(depth_operations)
                            } else {
                                None
                            },
                            stencil_ops: None,
                        }
                    }),
                });

        pipelines.iter().enumerate().for_each(|(i, pipeline)| {
            inox_profiler::scoped_profile!("pipeline[{}]::draw", pipeline.name());

            let pipeline_id = pipelines_id[i];
            let instance_count = graphics_data.instance_count(pipeline_id);
            if instance_count > 0 && pipeline.is_initialized() {
                render_pass.set_bind_group(0, pipeline.binding_data().bind_group(), &[]);
                render_pass.set_bind_group(1, render_pass_context.texture_bind_group, &[]);

                render_pass.set_pipeline(pipeline.render_pipeline());

                if let Some(buffer_slice) = graphics_data.vertex_buffer(pipeline_id) {
                    render_pass.set_vertex_buffer(0, buffer_slice);
                }
                if let Some(instance_buffer) = graphics_data.instance_buffer(pipeline_id) {
                    render_pass.set_vertex_buffer(1, instance_buffer);
                }
                if let Some(buffer_slice) = graphics_data.index_buffer(pipeline_id) {
                    render_pass.set_index_buffer(buffer_slice, wgpu::IndexFormat::Uint32);
                }

                if graphics_data.index_count(pipeline_id) > 0 {
                    if is_indirect_mode_enabled() && self.data.render_mode == RenderMode::Indirect {
                        let commands_count = graphics_data.commands_count(pipeline_id);
                        if let Some(command_buffer) = graphics_data.commands_buffer(pipeline_id) {
                            render_pass.multi_draw_indexed_indirect(
                                command_buffer,
                                0,
                                commands_count as u32,
                            );
                        }
                    } else {
                        graphics_data.for_each_instance(
                            pipeline_id,
                            |_mesh_id, index, instance_data, vertices_range, indices_range| {
                                let x = (instance_data.draw_area[0] as u32)
                                    .clamp(0, render_pass_context.context.config.width);
                                let y = (instance_data.draw_area[1] as u32)
                                    .clamp(0, render_pass_context.context.config.height);
                                let width = (instance_data.draw_area[2] as u32)
                                    .clamp(0, render_pass_context.context.config.width - x);
                                let height = (instance_data.draw_area[3] as u32)
                                    .clamp(0, render_pass_context.context.config.height - y);

                                render_pass.set_scissor_rect(x, y, width, height);
                                render_pass.draw_indexed(
                                    indices_range.start as _..(indices_range.end + 1) as _,
                                    vertices_range.start as _,
                                    index as _..(index as u32 + 1),
                                );
                            },
                        );
                    }
                } else {
                    render_pass.draw(
                        0..(graphics_data.vertex_count(pipeline_id) + 1) as u32,
                        0..instance_count as u32,
                    );
                }
            }
        });
    }
}
