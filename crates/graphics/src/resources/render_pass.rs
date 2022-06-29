use std::{collections::HashMap, path::Path};

use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedData, SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file};
use inox_uid::generate_random_uid;

use crate::{
    BindingData, GpuBuffer, LoadOperation, RenderContext, RenderMode, RenderPassData,
    RenderPipeline, RenderTarget, StoreOperation, Texture, TextureData, TextureId,
    VertexBufferLayoutBuilder,
};

pub type RenderPassId = ResourceId;

#[derive(Clone)]
pub struct RenderPass {
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    name: String,
    load_color: LoadOperation,
    store_color: StoreOperation,
    load_depth: LoadOperation,
    store_depth: StoreOperation,
    render_mode: RenderMode,
    pipeline: Handle<RenderPipeline>,
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
        self.invalidate();
    }
}

impl DataTypeResource for RenderPass {
    type DataType = RenderPassData;
    type OnCreateData = <Self as ResourceTrait>::OnCreateData;

    fn new(_id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            name: String::new(),
            load_color: LoadOperation::DontCare,
            store_color: StoreOperation::DontCare,
            load_depth: LoadOperation::DontCare,
            store_depth: StoreOperation::DontCare,
            render_mode: RenderMode::Indirect,
            pipeline: None,
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
        data: &Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let mut pass = Self {
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            name: data.name.clone(),
            load_color: data.load_color,
            store_color: data.store_color,
            load_depth: data.load_depth,
            store_depth: data.store_depth,
            render_mode: data.render_mode,
            pipeline: None,
            render_texture: None,
            depth_texture: None,
            is_initialized: false,
        };
        pass.render_target(data.render_target)
            .depth_target(data.depth_target)
            .set_pipeline(&data.pipeline);
        pass
    }
}

impl RenderPass {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn pipeline(&self) -> &Resource<RenderPipeline> {
        self.pipeline.as_ref().unwrap()
    }
    pub fn set_pipeline(&mut self, path: &Path) -> &mut Self {
        if !path.as_os_str().is_empty() {
            let pipeline =
                RenderPipeline::request_load(&self.shared_data, &self.message_hub, path, None);
            self.pipeline = Some(pipeline);
        };
        self
    }
    pub fn render_target(&mut self, render_target: RenderTarget) -> &mut Self {
        self.render_texture = match render_target {
            RenderTarget::Texture {
                width,
                height,
                format,
                read_back: _,
            } => {
                let texture_id = generate_random_uid();
                let image_data = Texture::create_from_format(width, height, format);
                let mut texture = Texture::create_from_data(
                    &self.shared_data,
                    &self.message_hub,
                    texture_id,
                    &TextureData {
                        width,
                        height,
                        format,
                        data: image_data,
                    },
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
        self.render_texture = Some(texture.clone());
        self
    }
    pub fn depth_target(&mut self, render_target: RenderTarget) -> &mut Self {
        self.depth_texture = match render_target {
            RenderTarget::Texture {
                width,
                height,
                format,
                read_back: _,
            } => {
                let texture_id = generate_random_uid();
                let image_data = Texture::create_from_format(width, height, format);
                let mut texture = Texture::create_from_data(
                    &self.shared_data,
                    &self.message_hub,
                    texture_id,
                    &TextureData {
                        width,
                        height,
                        format,
                        data: image_data,
                    },
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
    pub fn set_load_color_operation(&mut self, load_color: LoadOperation) -> &mut Self {
        self.load_color = load_color;
        self
    }
    pub fn set_store_color_operation(&mut self, store_color: StoreOperation) -> &mut Self {
        self.store_color = store_color;
        self
    }
    pub fn set_load_depth_operation(&mut self, load_depth: LoadOperation) -> &mut Self {
        self.load_depth = load_depth;
        self
    }
    pub fn set_store_depth_operation(&mut self, store_depth: StoreOperation) -> &mut Self {
        self.store_depth = store_depth;
        self
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

    pub fn init_pipeline(
        &mut self,
        render_context: &mut RenderContext,
        binding_data: &BindingData,
        vertex_layout: VertexBufferLayoutBuilder,
        instance_layout: VertexBufferLayoutBuilder,
    ) {
        let render_format = render_context.render_format(self);
        let depth_format = render_context.depth_format(self);
        if let Some(pipeline) = &self.pipeline {
            pipeline.get_mut().init(
                render_context,
                render_format,
                depth_format,
                binding_data,
                vertex_layout,
                instance_layout,
            );
        }
    }
    pub fn color_operations(&self) -> wgpu::Operations<wgpu::Color> {
        wgpu::Operations {
            load: match &self.load_color {
                LoadOperation::Load => wgpu::LoadOp::Load,
                _ => wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            },
            store: matches!(&self.store_color, StoreOperation::Store),
        }
    }

    pub fn depth_operations(&self) -> wgpu::Operations<f32> {
        wgpu::Operations {
            load: match &self.load_depth {
                LoadOperation::Load => wgpu::LoadOp::Load,
                _ => wgpu::LoadOp::Clear(1.),
            },
            store: matches!(&self.store_depth, StoreOperation::Store),
        }
    }

    pub fn begin<'a>(
        &'a self,
        render_context: &'a RenderContext,
        binding_data: &'a BindingData,
        buffers: &'a HashMap<ResourceId, GpuBuffer>,
        pipeline: &'a RenderPipeline,
        encoder: &'a mut wgpu::CommandEncoder,
    ) -> wgpu::RenderPass<'a> {
        let render_target = render_context.render_target(self);
        let depth_target = render_context.depth_target(self);

        let color_operations = self.color_operations();
        let depth_write_enabled = pipeline.data().depth_write_enabled;

        let label = format!("RenderPass {}", self.name);
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

        render_pass.set_pipeline(pipeline.render_pipeline());

        let num_vertex_buffers = binding_data.vertex_buffers_count();
        for i in 0..num_vertex_buffers {
            let id = binding_data.vertex_buffer(i);
            if let Some(buffer) = buffers.get(id) {
                render_pass.set_vertex_buffer(i as _, buffer.gpu_buffer().unwrap().slice(..));
            }
        }

        let index_id = binding_data.index_buffer();
        if let Some(buffer) = buffers.get(index_id) {
            render_pass.set_index_buffer(
                buffer.gpu_buffer().unwrap().slice(..),
                crate::IndexFormat::U32.into(),
            );
        }

        render_pass
    }

    pub fn draw_meshlets(&self, render_context: &RenderContext, mut render_pass: wgpu::RenderPass) {
        let mesh_flags = self.pipeline().get().data().mesh_flags;
        if let Some(instances) = render_context.render_buffers.instances.get(&mesh_flags) {
            let meshlets = render_context.render_buffers.meshlets.data();
            instances.for_each_entry(|index, instance| {
                /*if let Some(mesh_id) = render_context
                .render_buffers
                .meshes
                .id_at(instance.mesh_index as _)*/
                {
                    let mesh = render_context
                        .render_buffers
                        .meshes
                        .at(instance.mesh_index as _);
                    for i in mesh.meshlet_offset..mesh.meshlet_offset + mesh.meshlet_count {
                        let meshlet = &meshlets[i as usize];
                        /*
                        if let Some(area) =
                            render_context.render_buffers.draw_area.get(&mesh_id)
                        {
                            let x =
                                (area[0] as u32).clamp(0, render_context.core.config.width);
                            let y = (area[1] as u32)
                                .clamp(0, render_context.core.config.height);
                            let width = (x + area[2] as u32)
                                .clamp(0, render_context.core.config.width - x);
                            let height = (y + area[3] as u32)
                                .clamp(0, render_context.core.config.height - y);
                            render_pass.set_scissor_rect(x, y, width, height);
                        }
                        */
                        render_pass.draw_indexed(
                            (mesh.indices_offset + meshlet.indices_offset) as _
                                ..(mesh.indices_offset
                                    + meshlet.indices_offset
                                    + meshlet.indices_count) as _,
                            mesh.vertex_offset as _,
                            index as _..(index as u32 + 1),
                        );
                    }
                }
            });
        }
    }
}
