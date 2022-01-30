use std::path::Path;

use image::DynamicImage;
use sabi_messenger::MessengerRw;
use sabi_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedData, SharedDataRc,
};
use sabi_serialize::{generate_random_uid, read_from_file};

use crate::{
    GraphicsMesh, LoadOperation, Pipeline, RenderContext, RenderMode, RenderPassData, RenderTarget,
    StoreOperation, Texture, TextureHandler,
};

pub type RenderPassId = ResourceId;

#[derive(Default, Clone)]
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
    pub graphics_mesh: &'a GraphicsMesh,
    pub bind_groups: &'a [&'a wgpu::BindGroup],
    pub bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
}

impl DataTypeResource for RenderPass {
    type DataType = RenderPassData;
    type OnCreateData = ();

    fn invalidate(&mut self) -> &mut Self {
        self.is_initialized = false;
        self
    }
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
    fn deserialize_data(path: &Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }
    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _id: &RenderPassId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(&mut self, _shared_data: &SharedData, _messenger: &MessengerRw, _id: &RenderPassId) {}

    fn create_from_data(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
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
                    Pipeline::request_load(shared_data, global_messenger, path.as_path(), None);
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
                let texture =
                    Texture::create_from_data(shared_data, global_messenger, id, image_data);
                let texture = shared_data.add_resource(global_messenger, generate_random_uid(), texture);
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
    pub fn init(&mut self, context: &RenderContext, texture_handler: &mut TextureHandler) {
        if self.is_initialized {
            return;
        }
        if let Some(texture) = &self.target_texture {
            if texture_handler.get_texture_atlas(texture.id()).is_none() {
                texture_handler.add_render_target(
                    context,
                    texture.id(),
                    texture.get().width(),
                    texture.get().height(),
                );
            }
        }
        self.is_initialized = true;
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

    pub fn draw(&mut self, render_pass_context: RenderPassDrawContext) {
        let graphics_mesh = render_pass_context.graphics_mesh;
        if graphics_mesh.vertex_count() == 0 {
            return;
        }
        let mut pipelines = self
            .pipelines
            .iter()
            .map(|h| h.get_mut())
            .collect::<Vec<_>>();
        let pipelines_id = self
            .pipelines
            .iter()
            .map(|h| h.id())
            .collect::<Vec<_>>();
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

        render_pass_context
            .bind_groups
            .iter()
            .enumerate()
            .for_each(|(i, &bind_group)| {
                render_pass.set_bind_group(i as u32, bind_group, &[]);
            });

        pipelines.iter_mut().enumerate().for_each(|(i, pipeline)| {
            let pipeline_id = pipelines_id[i];
            if let Some(instance_data) =  graphics_mesh.instances(pipeline_id) {
                let instance_count = instance_data.len();
                {
                    pipeline.init(
                        render_pass_context.context,
                        render_pass_context.bind_group_layouts,
                        render_pass_context.format,
                    );
                }
                if pipeline.is_initialized() {
                    render_pass.set_pipeline(pipeline.render_pipeline());
                    if let Some(buffer_slice) = graphics_mesh.vertex_buffer() {
                        render_pass.set_vertex_buffer(0, buffer_slice);
                    }
                    if let Some(instance_buffer) = graphics_mesh.instance_buffer(pipeline_id) {
                        render_pass.set_vertex_buffer(1, instance_buffer);
                    }
                    if let Some(buffer_slice) = render_pass_context.graphics_mesh.index_buffer() {
                        render_pass.set_index_buffer(buffer_slice, wgpu::IndexFormat::Uint32);
                    }

                    if render_pass_context.graphics_mesh.index_count() > 0 {
                        if self.data.render_mode == RenderMode::Indirect {
                            if let Some(indirect_buffer) = graphics_mesh.indirect_buffer(pipeline_id) {
                                render_pass.multi_draw_indexed_indirect(
                                    indirect_buffer,
                                    0,
                                    instance_count as u32,
                                );
                            }
                        } else if let Some(instance_data) = graphics_mesh.instances(pipeline_id) {
                            for i in 0..instance_count as u32 {
                                if let Some(index) = instance_data.iter().position(|instance| instance.id == i) {
                                    let instance_data = instance_data[index];
                                    if let Some(indirect_command) = graphics_mesh.indirect(index, pipeline_id) {
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
                                            indirect_command.base_index
                                                ..(indirect_command.base_index
                                                    + indirect_command.vertex_count)
                                                    as _,
                                            indirect_command.vertex_offset as _,
                                            (index as u32)..(index as u32 + 1),
                                        )
                                    }
                                } else {
                                    eprintln!("Unable to find instance {} for pipeline {} - did you forget to assign draw_index to meshes?", i, pipeline.name());
                                }
                            } 
                        }
                    } else {
                        render_pass.draw(
                            0..render_pass_context.graphics_mesh.vertex_count() as u32,
                            0..instance_count as u32,
                        );
                    }
                }
            }
        });
    }
}
