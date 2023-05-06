use std::{collections::HashMap, ops::Range, path::Path};

use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedDataRc,
};

use crate::{
    gpu_texture::GpuTexture, platform::is_indirect_mode_enabled, AsBinding, BindingData, BufferId,
    CommandBuffer, DrawCommandType, GpuBuffer, LoadOperation, RenderContext, RenderCoreContextRc,
    RenderMode, RenderPassData, RenderPipeline, RenderTarget, StoreOperation, Texture, TextureId,
    TextureUsage, TextureView, VertexBufferLayoutBuilder,
};

pub type RenderPassId = ResourceId;

pub struct RenderPassBeginData<'a> {
    pub render_core_context: &'a RenderCoreContextRc,
    pub render_targets: &'a [GpuTexture],
    pub buffers: &'a HashMap<BufferId, GpuBuffer>,
    pub surface_view: &'a TextureView,
    pub command_buffer: &'a mut CommandBuffer,
}

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
    fn invalidate(&mut self) -> &mut Self {
        self
    }
    fn is_initialized(&self) -> bool {
        true
    }
}

impl DataTypeResource for RenderPass {
    type DataType = RenderPassData;

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
        } = render_target
        {
            let texture = Texture::create_from_format(
                &self.shared_data,
                &self.message_hub,
                width,
                height,
                format,
                TextureUsage::TextureBinding
                    | TextureUsage::CopySrc
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
            } => {
                let texture = Texture::create_from_format(
                    &self.shared_data,
                    &self.message_hub,
                    width,
                    height,
                    format,
                    TextureUsage::TextureBinding
                        | TextureUsage::CopySrc
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
        render_context: &RenderContext,
        binding_data: &mut BindingData,
        vertex_layout: Option<VertexBufferLayoutBuilder>,
        instance_layout: Option<VertexBufferLayoutBuilder>,
    ) {
        let render_targets = render_context.texture_handler.render_targets();

        let mut render_formats = Vec::new();
        let render_textures = self.render_textures_id();
        render_textures.iter().for_each(|&id| {
            if let Some(texture) = render_targets.iter().find(|t| t.id() == id) {
                render_formats.push(texture.format());
            }
        });

        let mut depth_format = None;
        if let Some(texture) = self.depth_texture() {
            if let Some(texture) = render_targets.iter().find(|t| t.id() == texture.id()) {
                depth_format = Some(texture.format());
            }
        }

        if let Some(pipeline) = &self.pipeline {
            binding_data.set_bind_group_layout();

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
        binding_data: &'a mut BindingData,
        pipeline: &'a RenderPipeline,
        render_pass_begin_data: RenderPassBeginData<'a>,
    ) -> wgpu::RenderPass<'a> {
        inox_profiler::scoped_profile!("render_pass::begin");
        inox_profiler::gpu_scoped_profile!(
            &mut render_pass_begin_data.command_buffer.encoder,
            &render_pass_begin_data.render_core_context.device,
            "render_pass::begin",
        );

        let mut render_targets_views = Vec::new();
        let mut depth_target_view = None;

        let render_textures = self.render_textures_id();
        if render_textures.is_empty() {
            render_targets_views.push(render_pass_begin_data.surface_view.as_wgpu());
        } else {
            render_textures.iter().for_each(|&id| {
                if let Some(texture) = render_pass_begin_data
                    .render_targets
                    .iter()
                    .find(|t| t.id() == id)
                {
                    render_targets_views.push(texture.view().as_wgpu());
                }
            });
        }
        if let Some(texture) = self.depth_texture() {
            if let Some(texture) = render_pass_begin_data
                .render_targets
                .iter()
                .find(|t| t.id() == texture.id())
            {
                depth_target_view = Some(texture.view().as_wgpu());
            }
        }

        let color_operations = self.color_operations();
        let depth_write_enabled = pipeline.data().depth_write_enabled;

        let label = format!("RenderPass {}", self.name);
        let mut render_pass = {
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass_begin_data.command_buffer.encoder,
                &render_pass_begin_data.render_core_context.device,
                "encoder::begin_render_pass",
            );
            render_pass_begin_data
                .command_buffer
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(label.as_str()),
                    color_attachments: render_targets_views
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
                    depth_stencil_attachment: depth_target_view.map(|depth_view| {
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
                })
        };
        {
            binding_data.set_bind_groups();

            binding_data
                .bind_groups()
                .iter()
                .enumerate()
                .for_each(|(index, bind_group)| {
                    inox_profiler::scoped_profile!("render_pass::bind_groups");
                    inox_profiler::gpu_scoped_profile!(
                        &mut render_pass,
                        &render_pass_begin_data.render_core_context.device,
                        "render_pass::bind_groups",
                    );
                    render_pass.set_bind_group(index as _, bind_group, &[]);
                });
        }
        {
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass,
                &render_pass_begin_data.render_core_context.device,
                "render_pass::set_pipeline",
            );
            render_pass.set_pipeline(pipeline.render_pipeline());
        }

        let num_vertex_buffers = binding_data.vertex_buffers_count();
        for i in 0..num_vertex_buffers {
            inox_profiler::scoped_profile!("render_pass::bind_vertex_buffer");
            let id = binding_data.vertex_buffer(i);
            if let Some(buffer) = render_pass_begin_data.buffers.get(id) {
                inox_profiler::gpu_scoped_profile!(
                    &mut render_pass,
                    &render_pass_begin_data.render_core_context.device,
                    "render_pass::set_vertex_buffer",
                );
                render_pass.set_vertex_buffer(i as _, buffer.gpu_buffer().unwrap().slice(..));
            }
        }

        if let Some(index_id) = binding_data.index_buffer() {
            if let Some(buffer) = render_pass_begin_data.buffers.get(index_id) {
                inox_profiler::scoped_profile!("render_pass::bind_index_buffer");
                inox_profiler::gpu_scoped_profile!(
                    &mut render_pass,
                    &render_pass_begin_data.render_core_context.device,
                    "render_pass::set_index_buffer",
                );
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
        let meshlets = render_context.render_buffers.meshlets.read().unwrap();
        let meshlets = meshlets.data();
        render_context
            .render_buffers
            .meshes
            .read()
            .unwrap()
            .for_each_id(|mesh_id, _, mesh| {
                if let Some(flags) = render_context
                    .render_buffers
                    .meshes_flags
                    .read()
                    .unwrap()
                    .get(mesh_id)
                {
                    if flags == &mesh_flags {
                        inox_profiler::scoped_profile!("render_pass::draw_mesh");
                        for i in mesh.meshlets_offset..mesh.meshlets_offset + mesh.meshlets_count {
                            inox_profiler::scoped_profile!("render_pass::draw_indexed");
                            inox_profiler::gpu_scoped_profile!(
                                &mut render_pass,
                                &render_context.core.device,
                                "render_pass::draw_indexed",
                            );
                            let meshlet = &meshlets[i as usize];
                            render_pass.draw_indexed(
                                (mesh.indices_offset + meshlet.indices_offset) as _
                                    ..(mesh.indices_offset
                                        + meshlet.indices_offset
                                        + meshlet.indices_count)
                                        as _,
                                mesh.vertex_offset as _,
                                i as _..(i + 1),
                            );
                        }
                    }
                }
            });
    }

    pub fn indirect_indexed_draw<'a>(
        &self,
        render_context: &RenderContext,
        buffers: &'a HashMap<BufferId, GpuBuffer>,
        draw_commands_type: DrawCommandType,
        mut render_pass: wgpu::RenderPass<'a>,
    ) {
        inox_profiler::scoped_profile!("render_pass::indirect_draw");

        if is_indirect_mode_enabled() && self.render_mode == RenderMode::Indirect {
            let mesh_flags = self.pipeline().get().data().mesh_flags;
            if let Some(commands) = render_context
                .render_buffers
                .commands
                .read()
                .unwrap()
                .get(&mesh_flags)
            {
                if let Some(commands) = commands.map.get(&draw_commands_type) {
                    if !commands.commands.is_empty() {
                        let commands_id = commands.commands.id();
                        if let Some(commands_buffer) = buffers.get(&commands_id) {
                            let count_id = commands.counter.id();
                            if let Some(count_buffer) = buffers.get(&count_id) {
                                inox_profiler::gpu_scoped_profile!(
                                    &mut render_pass,
                                    &render_context.core.device,
                                    "render_pass::multi_draw_indexed_indirect_count",
                                );
                                render_pass.multi_draw_indexed_indirect_count(
                                    commands_buffer.gpu_buffer().unwrap(),
                                    0,
                                    count_buffer.gpu_buffer().unwrap(),
                                    0,
                                    commands.commands.item_count() as _,
                                );
                                return;
                            }
                        }
                    }
                }
            }
        }
        //TODO: use debug_log_once
        //inox_log::debug_log!("Unable to use indirect_draw - using normal draw_indexed");
        self.draw_meshlets(render_context, render_pass);
    }

    pub fn draw(
        &self,
        render_context: &RenderContext,
        mut render_pass: wgpu::RenderPass,
        vertices: Range<u32>,
        instances: Range<u32>,
    ) {
        inox_profiler::scoped_profile!("render_pass::draw");
        inox_profiler::gpu_scoped_profile!(
            &mut render_pass,
            &render_context.core.device,
            "render_pass::draw",
        );
        render_pass.draw(vertices, instances);
    }

    pub fn draw_meshes(&self, render_context: &RenderContext, mut render_pass: wgpu::RenderPass) {
        inox_profiler::scoped_profile!("render_pass::draw_meshes");

        let mesh_flags = self.pipeline().get().data().mesh_flags;
        let meshlets = render_context.render_buffers.meshlets.read().unwrap();
        let meshlets = meshlets.data();
        render_context
            .render_buffers
            .meshes
            .read()
            .unwrap()
            .for_each_id(|mesh_id, index, mesh| {
                if let Some(flags) = render_context
                    .render_buffers
                    .meshes_flags
                    .read()
                    .unwrap()
                    .get(mesh_id)
                {
                    if flags == &mesh_flags {
                        let start = mesh.indices_offset;
                        let mut end = start;
                        for i in mesh.meshlets_offset..mesh.meshlets_offset + mesh.meshlets_count {
                            let meshlet = &meshlets[i as usize];
                            end += meshlet.indices_count;
                        }
                        inox_profiler::gpu_scoped_profile!(
                            &mut render_pass,
                            &render_context.core.device,
                            "render_pass::draw_indexed",
                        );
                        render_pass.draw_indexed(
                            start..end as _,
                            mesh.vertex_offset as _,
                            index as _..(index as u32 + 1),
                        );
                    }
                }
            });
    }
}
