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
    platform::is_indirect_mode_enabled, BindingData, IndexFormat, LoadOperation, RenderContext,
    RenderMode, RenderPassData, RenderPipeline, RenderTarget, StoreOperation, Texture, TextureId,
};

pub type RenderPassId = ResourceId;

#[derive(Clone)]
pub struct RenderPass {
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    data: RenderPassData,
    pipelines: Vec<Resource<RenderPipeline>>,
    render_texture: Handle<Texture>,
    depth_texture: Handle<Texture>,
    is_initialized: bool,
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
            .set_pipelines(pipelines);
        pass
    }
}

impl RenderPass {
    pub fn data(&self) -> &RenderPassData {
        &self.data
    }
    pub fn pipelines(&self) -> &[Resource<RenderPipeline>] {
        self.pipelines.as_slice()
    }
    pub fn set_pipelines(&mut self, pipelines: Vec<PathBuf>) -> &mut Self {
        self.pipelines.clear();
        pipelines.iter().for_each(|path| {
            if !path.as_os_str().is_empty() {
                let pipeline = RenderPipeline::request_load(
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
    pub fn render_texture_id(&self) -> Option<&TextureId> {
        self.render_texture.as_ref().map(|t| t.id())
    }
    pub fn depth_texture_id(&self) -> Option<&TextureId> {
        self.depth_texture.as_ref().map(|t| t.id())
    }
    pub fn init(&mut self, render_context: &mut RenderContext) {
        if !self.is_initialized {
            if let Some(texture) = &self.render_texture {
                let texture_handler = &mut render_context.texture_handler;
                if texture_handler.get_texture_atlas(texture.id()).is_none() {
                    texture_handler.add_custom_texture(
                        &render_context.core.device,
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
                let texture_handler = &mut render_context.texture_handler;
                if texture_handler.get_texture_atlas(texture.id()).is_none() {
                    texture_handler.add_custom_texture(
                        &render_context.core.device,
                        texture.id(),
                        texture.get().width(),
                        texture.get().height(),
                        wgpu::TextureFormat::Depth32Float,
                        wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::COPY_DST
                            | wgpu::TextureUsages::RENDER_ATTACHMENT,
                    );
                }
            }
            self.is_initialized = true;
        }
    }

    pub fn init_pipelines(
        &mut self,
        render_context: &mut RenderContext,
        binding_data: &BindingData,
    ) {
        let render_format = render_context.render_format(self);
        let depth_format = render_context.depth_format(self);
        self.pipelines.iter().for_each(|pipeline| {
            pipeline
                .get_mut()
                .init(render_context, render_format, depth_format, binding_data);
        });
    }
    pub fn color_operations(&self) -> wgpu::Operations<wgpu::Color> {
        wgpu::Operations {
            load: match &self.data.load_color {
                LoadOperation::Load => wgpu::LoadOp::Load,
                _ => wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            },
            store: matches!(&self.data.store_color, StoreOperation::Store),
        }
    }

    pub fn depth_operations(&self) -> wgpu::Operations<f32> {
        wgpu::Operations {
            load: match &self.data.load_depth {
                LoadOperation::Load => wgpu::LoadOp::Load,
                _ => wgpu::LoadOp::Clear(1.),
            },
            store: matches!(&self.data.store_depth, StoreOperation::Store),
        }
    }

    pub fn begin<'a>(
        &'a self,
        render_context: &'a RenderContext,
        binding_data: &'a BindingData,
        encoder: &'a mut wgpu::CommandEncoder,
    ) -> wgpu::RenderPass<'a> {
        let render_target = render_context.render_target(self);
        let depth_target = render_context.depth_target(self);

        let color_operations = self.color_operations();
        let mut depth_write_enabled = false;
        let pipelines = self.pipelines().iter().map(|h| h.get()).collect::<Vec<_>>();
        pipelines.iter().for_each(|pipeline| {
            depth_write_enabled |= pipeline.data().depth_write_enabled;
        });

        let label = format!("RenderPass {}", self.data().name);
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label.as_str()),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: render_target,
                resolve_target: None,
                ops: color_operations,
            }],
            depth_stencil_attachment: depth_target.map(|depth_view| {
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

        binding_data
            .bind_groups()
            .iter()
            .enumerate()
            .for_each(|(index, bind_group)| {
                render_pass.set_bind_group(index as _, bind_group, &[]);
            });

        render_pass
    }

    pub fn draw(&self, render_context: &RenderContext, render_pass: wgpu::RenderPass) {
        let pipelines = self.pipelines().iter().map(|h| h.get()).collect::<Vec<_>>();
        {
            let graphics_data = render_context.graphics_data.get();
            let mut render_pass = render_pass;
            pipelines.iter().enumerate().for_each(|(i, pipeline)| {
                let pipeline_id = self.pipelines[i].id();
                let instance_count = graphics_data.instance_count(pipeline_id);
                if instance_count > 0 && pipeline.is_initialized() {
                    render_pass.set_pipeline(pipeline.render_pipeline());

                    if let Some(buffer_slice) = graphics_data.vertex_buffer(pipeline_id) {
                        render_pass.set_vertex_buffer(0, buffer_slice);
                    }
                    if let Some(instance_buffer) = graphics_data.instance_buffer(pipeline_id) {
                        render_pass.set_vertex_buffer(1, instance_buffer);
                    }
                    if let Some(buffer_slice) = graphics_data.index_buffer(pipeline_id) {
                        render_pass.set_index_buffer(buffer_slice, IndexFormat::U32.into());
                    }

                    if graphics_data.index_count(pipeline_id) > 0 {
                        if is_indirect_mode_enabled()
                            && self.data().render_mode == RenderMode::Indirect
                        {
                            let commands_count = graphics_data.commands_count(pipeline_id);
                            if let Some(command_buffer) = graphics_data.commands_buffer(pipeline_id)
                            {
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
                                        .clamp(0, render_context.core.config.width);
                                    let y = (instance_data.draw_area[1] as u32)
                                        .clamp(0, render_context.core.config.height);
                                    let width = (instance_data.draw_area[2] as u32)
                                        .clamp(0, render_context.core.config.width - x);
                                    let height = (instance_data.draw_area[3] as u32)
                                        .clamp(0, render_context.core.config.height - y);

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
}
