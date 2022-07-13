use std::{collections::HashMap, ops::Range, path::Path};

use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedData, SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file};

use crate::{
    platform::is_indirect_mode_enabled, AsBinding, BindingData, BufferId, CommandBuffer,
    DrawCommand, GpuBuffer, LoadOperation, RenderContext, RenderMode, RenderPassData,
    RenderPipeline, RenderTarget, StoreOperation, Texture, TextureId, TextureUsage,
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
    render_textures: Vec<Resource<Texture>>,
    depth_texture: Handle<Texture>,
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
            render_textures: Vec::new(),
            depth_texture: None,
        }
    }
    fn invalidate(&mut self) -> &mut Self {
        self
    }
    fn is_initialized(&self) -> bool {
        true
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
            render_textures: Vec::new(),
            depth_texture: None,
        };
        pass.add_render_target(data.render_target)
            .add_depth_target(data.depth_target)
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
    pub fn add_render_target(&mut self, render_target: RenderTarget) -> &mut Self {
        if let RenderTarget::Texture {
            width,
            height,
            format,
            read_back: _,
        } = render_target
        {
            let texture = Texture::create_from_format(
                &self.shared_data,
                &self.message_hub,
                width,
                height,
                format,
                TextureUsage::TextureBinding
                    | TextureUsage::CopyDst
                    | TextureUsage::RenderAttachment,
            );
            self.render_textures.push(texture)
        }

        self
    }
    pub fn add_render_target_from_texture(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.render_textures.push(texture.clone());
        self
    }
    pub fn add_depth_target(&mut self, render_target: RenderTarget) -> &mut Self {
        self.depth_texture = match render_target {
            RenderTarget::Texture {
                width,
                height,
                format,
                read_back: _,
            } => {
                let texture = Texture::create_from_format(
                    &self.shared_data,
                    &self.message_hub,
                    width,
                    height,
                    format,
                    TextureUsage::TextureBinding
                        | TextureUsage::CopyDst
                        | TextureUsage::RenderAttachment,
                );
                Some(texture)
            }
            _ => None,
        };
        self
    }
    pub fn add_depth_target_from_texture(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.depth_texture = Some(texture.clone());
        self
    }
    pub fn render_textures(&self) -> &[Resource<Texture>] {
        &self.render_textures
    }
    pub fn depth_texture(&self) -> &Handle<Texture> {
        &self.depth_texture
    }
    pub fn render_textures_id(&self) -> Vec<&TextureId> {
        self.render_textures.iter().map(|t| t.id()).collect()
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

    pub fn init(
        &mut self,
        render_context: &mut RenderContext,
        binding_data: &BindingData,
        vertex_layout: Option<VertexBufferLayoutBuilder>,
        instance_layout: Option<VertexBufferLayoutBuilder>,
    ) {
        let render_formats = render_context.render_formats(self);
        let depth_format = render_context.depth_format(self);
        if let Some(pipeline) = &self.pipeline {
            pipeline.get_mut().init(
                render_context,
                render_formats,
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
        buffers: &'a HashMap<BufferId, GpuBuffer>,
        pipeline: &'a RenderPipeline,
        command_buffer: &'a mut CommandBuffer,
    ) -> wgpu::RenderPass<'a> {
        inox_profiler::scoped_profile!("render_pass::begin");

        let render_targets = render_context.render_targets(self);
        let depth_target = render_context.depth_target(self);

        let color_operations = self.color_operations();
        let depth_write_enabled = pipeline.data().depth_write_enabled;

        let label = format!("RenderPass {}", self.name);
        let mut render_pass =
            command_buffer
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(label.as_str()),
                    color_attachments: render_targets
                        .iter()
                        .map(|&render_target| {
                            Some(wgpu::RenderPassColorAttachment {
                                view: render_target,
                                resolve_target: None,
                                ops: color_operations,
                            })
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
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
                inox_profiler::scoped_profile!("render_pass::bind_groups");
                render_pass.set_bind_group(index as _, bind_group, &[]);
            });

        render_pass.set_pipeline(pipeline.render_pipeline());

        let num_vertex_buffers = binding_data.vertex_buffers_count();
        for i in 0..num_vertex_buffers {
            inox_profiler::scoped_profile!("render_pass::bind_vertex_buffer");
            let id = binding_data.vertex_buffer(i);
            if let Some(buffer) = buffers.get(id) {
                render_pass.set_vertex_buffer(i as _, buffer.gpu_buffer().unwrap().slice(..));
            }
        }

        if let Some(index_id) = binding_data.index_buffer() {
            if let Some(buffer) = buffers.get(index_id) {
                inox_profiler::scoped_profile!("render_pass::bind_index_buffer");
                render_pass.set_index_buffer(
                    buffer.gpu_buffer().unwrap().slice(..),
                    crate::IndexFormat::U32.into(),
                );
            }
        }

        render_pass
    }

    pub fn draw_meshlets(&self, render_context: &RenderContext, mut render_pass: wgpu::RenderPass) {
        inox_profiler::scoped_profile!("render_pass::draw_meshlets");

        let mesh_flags = self.pipeline().get().data().mesh_flags;
        if let Some(instances) = render_context.render_buffers.instances.get(&mesh_flags) {
            let meshlets = render_context.render_buffers.meshlets.data();
            instances.for_each_entry(|index, instance| {
                inox_profiler::scoped_profile!("render_pass::draw_instance");
                {
                    let mesh = render_context
                        .render_buffers
                        .meshes
                        .at(instance.mesh_index as _);
                    for i in mesh.meshlet_offset..mesh.meshlet_offset + mesh.meshlet_count {
                        inox_profiler::scoped_profile!("render_pass::draw_indexed");
                        let meshlet = &meshlets[i as usize];
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

    pub fn indirect_draw<'a>(
        &self,
        render_context: &RenderContext,
        buffers: &'a HashMap<BufferId, GpuBuffer>,
        mut render_pass: wgpu::RenderPass<'a>,
    ) {
        inox_profiler::scoped_profile!("render_pass::indirect_draw");

        if is_indirect_mode_enabled() && self.render_mode == RenderMode::Indirect {
            let mesh_flags = self.pipeline().get().data().mesh_flags;
            if let Some(commands) = render_context.render_buffers.commands.get(&mesh_flags) {
                let commands_count = commands.size() / std::mem::size_of::<DrawCommand>() as u64;
                let commands_id = commands.id();
                if let Some(buffer) = buffers.get(&commands_id) {
                    render_pass.multi_draw_indexed_indirect(
                        buffer.gpu_buffer().unwrap(),
                        0,
                        commands_count as _,
                    );
                    return;
                }
            }
        }
        inox_log::debug_log!("Unable to use indirect_draw - using normal draw_indexed");
        self.draw_meshlets(render_context, render_pass);
    }

    pub fn draw(
        &self,
        mut render_pass: wgpu::RenderPass,
        vertices: Range<u32>,
        instances: Range<u32>,
    ) {
        inox_profiler::scoped_profile!("render_pass::draw");

        render_pass.draw(vertices, instances);
    }
}
