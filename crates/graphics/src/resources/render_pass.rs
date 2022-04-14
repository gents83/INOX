use image::DynamicImage;
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedData, SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file};

use crate::{
    platform::is_indirect_mode_enabled, DataBuffer, GraphicsData, LoadOperation, Pipeline,
    RenderContext, RenderMode, RenderPassData, RenderTarget, StoreOperation, Texture,
};

pub type RenderPassId = ResourceId;

#[derive(Clone)]
pub struct RenderPass {
    data: RenderPassData,
    pipelines: Vec<Resource<Pipeline>>,
    target_texture: Handle<Texture>,
    is_initialized: bool,
}

pub struct RenderPassDrawContext<'a> {
    pub context: &'a RenderContext,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub texture_view: &'a wgpu::TextureView,
    pub format: &'a wgpu::TextureFormat,
    pub graphics_mesh: &'a Resource<GraphicsData>,
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

    fn new(_id: ResourceId, _shared_data: &SharedDataRc, _message_hub: &MessageHubRc) -> Self {
        Self {
            data: RenderPassData::default(),
            pipelines: Vec::new(),
            target_texture: None,
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
        id: ResourceId,
        data: Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let mut pipelines = Vec::new();
        data.pipelines.iter().for_each(|path| {
            if !path.as_os_str().is_empty() {
                let pipeline =
                    Pipeline::request_load(shared_data, message_hub, path.as_path(), None);
                pipelines.push(pipeline);
            };
        });

        let render_target = match &data.render_target {
            RenderTarget::Texture {
                width,
                height,
                read_back: _,
            } => {
                let image = DynamicImage::new_rgba8(*width, *height);
                let image_data = image.to_rgba8();
                let mut texture =
                    Texture::create_from_data(shared_data, message_hub, id, image_data);
                texture.on_create(shared_data, message_hub, &id, None);
                let texture = shared_data.add_resource(message_hub, id, texture);
                Some(texture)
            }
            _ => None,
        };

        Self {
            data,
            pipelines,
            target_texture: render_target,
            is_initialized: false,
        }
    }
}

impl RenderPass {
    pub fn data(&self) -> &RenderPassData {
        &self.data
    }
    pub fn render_target(&self) -> &Handle<Texture> {
        &self.target_texture
    }
    pub fn init(&mut self, context: &mut RenderContext) {
        if self.is_initialized {
            return;
        }
        if let Some(texture) = &self.target_texture {
            let texture_handler = &mut context.texture_handler;
            if texture_handler.get_texture_atlas(texture.id()).is_none() {
                texture_handler.add_render_target(
                    &context.device,
                    texture.id(),
                    texture.get().width(),
                    texture.get().height(),
                );
                self.is_initialized = true;
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

    fn depth_operations(&self, c: wgpu::Color) -> wgpu::Operations<wgpu::Color> {
        wgpu::Operations {
            load: match &self.data.load_depth {
                LoadOperation::Load => wgpu::LoadOp::Load,
                _ => wgpu::LoadOp::Clear(c),
            },
            store: matches!(&self.data.store_depth, StoreOperation::Store),
        }
    }

    pub fn prepare(
        &mut self,
        render_context: &RenderContext,
        graphics_mesh: &mut GraphicsData,
        format: &wgpu::TextureFormat,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        constant_data_buffer: &DataBuffer,
        dynamic_data_buffer: &DataBuffer,
    ) {
        if graphics_mesh.total_vertex_count() == 0 {
            return;
        }
        self.pipelines.iter_mut().for_each(|pipeline| {
            let pipeline_id = pipeline.id();
            let instance_count = graphics_mesh.instance_count(pipeline_id);
            if instance_count > 0 {
                if !pipeline.get_mut().init(
                    render_context,
                    texture_bind_group_layout,
                    format,
                    constant_data_buffer,
                    dynamic_data_buffer,
                ) {
                    return;
                }
                if is_indirect_mode_enabled() && self.data.render_mode == RenderMode::Indirect {
                    graphics_mesh.fill_command_buffer(render_context, pipeline_id);
                }
            }
        });
    }

    pub fn draw(&self, render_pass_context: RenderPassDrawContext) {
        let graphics_mesh = render_pass_context.graphics_mesh.get();
        if graphics_mesh.total_vertex_count() == 0 || graphics_mesh.total_index_count() == 0 {
            return;
        }
        let pipelines = self.pipelines.iter().map(|h| h.get()).collect::<Vec<_>>();
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
                    depth_stencil_attachment: None,
                });

        pipelines.iter().enumerate().for_each(|(i, pipeline)| {
            let pipeline_id = pipelines_id[i];
            let instance_count = graphics_mesh.instance_count(pipeline_id);
            if instance_count > 0 && pipeline.is_initialized() {
                render_pass.set_bind_group(0, pipeline.binding_data().bind_group(), &[]);
                render_pass.set_bind_group(1, render_pass_context.texture_bind_group, &[]);

                render_pass.set_pipeline(pipeline.render_pipeline());

                if let Some(buffer_slice) = graphics_mesh.vertex_buffer(pipeline_id) {
                    render_pass.set_vertex_buffer(0, buffer_slice);
                }
                if let Some(instance_buffer) = graphics_mesh.instance_buffer(pipeline_id) {
                    render_pass.set_vertex_buffer(1, instance_buffer);
                }
                if let Some(buffer_slice) = graphics_mesh.index_buffer(pipeline_id) {
                    render_pass.set_index_buffer(buffer_slice, wgpu::IndexFormat::Uint32);
                }

                if graphics_mesh.index_count(pipeline_id) > 0 {
                    if is_indirect_mode_enabled() && self.data.render_mode == RenderMode::Indirect {
                        let commands_count = graphics_mesh.commands_count(pipeline_id);
                        if let Some(command_buffer) = graphics_mesh.commands_buffer(pipeline_id) {
                            render_pass.multi_draw_indexed_indirect(
                                command_buffer,
                                0,
                                commands_count as u32,
                            );
                        }
                    } else {
                        graphics_mesh.for_each_instance(
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
                                    indices_range.start as _..indices_range.end as _,
                                    vertices_range.start as _,
                                    index as _..(index as u32 + 1),
                                );
                            },
                        );
                    }
                } else {
                    render_pass.draw(
                        0..graphics_mesh.vertex_count(pipeline_id) as u32,
                        0..instance_count as u32,
                    );
                }
            }
        });
    }
}
