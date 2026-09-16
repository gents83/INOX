use std::{
    collections::HashMap,
    ops::Range,
    path::Path,
    sync::{Arc, Mutex},
};

use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedDataRc,
};

use crate::{
    platform::has_multisampling_support, texture_ref::TextureRef, AsBinding, BindingData, BufferId,
    BufferRef, CommandBuffer, DrawIndexedCommand, GPUMesh, GPUMeshlet, GPUVector, LoadOperation,
    MeshFlags, RenderContext, RenderMode, RenderPassData, RenderPipeline, RenderTarget,
    StoreOperation, Texture, TextureId, TextureUsage, TextureView, VertexBufferLayoutBuilder,
    WebGpuContextRc, DRAW_INDEXED_COMMAND_OFFSET,
};

pub type RenderPassId = ResourceId;

pub struct RenderPassBeginData<'a> {
    pub render_core_context: &'a WebGpuContextRc,
    pub render_targets: &'a [TextureRef],
    pub buffers: &'a HashMap<BufferId, BufferRef>,
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
    meshlet_commands: Arc<Mutex<Vec<DrawIndexedCommand>>>,
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
            meshlet_commands: Arc::new(Mutex::new(Vec::new())),
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
            meshlet_commands: Arc::new(Mutex::new(Vec::new())),
        };
        pass.create_render_target(data.render_target)
            .create_depth_target(data.depth_target)
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
    pub fn create_render_target(&mut self, render_target: RenderTarget) -> &mut Self {
        if let RenderTarget::Texture {
            width,
            height,
            format,
            sample_count,
            mips_count,
        } = render_target
        {
            let texture = Texture::create_from_format(
                &self.shared_data,
                &self.message_hub,
                (width, height, sample_count, 1, mips_count),
                format,
                TextureUsage::TextureBinding
                    | TextureUsage::CopySrc
                    | TextureUsage::CopyDst
                    | TextureUsage::RenderTarget,
            );
            self.render_textures.push(texture)
        }

        self
    }
    pub fn remove_render_target(&mut self, texture_id: &TextureId) -> &mut Self {
        self.render_textures.retain(|t| t.id() != texture_id);
        self
    }
    pub fn remove_all_render_targets(&mut self) -> &mut Self {
        self.render_textures.clear();
        self
    }
    pub fn add_render_target(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.render_textures.push(texture.clone());
        self
    }
    pub fn create_depth_target(&mut self, render_target: RenderTarget) -> &mut Self {
        self.depth_texture = match render_target {
            RenderTarget::Texture {
                width,
                height,
                format,
                sample_count,
                mips_count,
            } => {
                let texture = Texture::create_from_format(
                    &self.shared_data,
                    &self.message_hub,
                    (width, height, sample_count, 1, mips_count),
                    format,
                    TextureUsage::TextureBinding
                        | TextureUsage::CopySrc
                        | TextureUsage::CopyDst
                        | TextureUsage::RenderTarget,
                );
                Some(texture)
            }
            _ => None,
        };
        self
    }
    pub fn add_depth_target(&mut self, texture: &Resource<Texture>) -> &mut Self {
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
        let (render_formats, depth_format) = {
            let render_targets = render_context.texture_handler().render_targets();
            let render_formats = self
                .render_textures
                .iter()
                .filter_map(|target| {
                    render_targets
                        .iter()
                        .find(|texture| texture.id() == target.id())
                        .map(|texture| *texture.format())
                })
                .collect::<Vec<_>>();
            let depth_format = self.depth_texture.as_ref().and_then(|target| {
                render_targets
                    .iter()
                    .find(|texture| texture.id() == target.id())
                    .map(|texture| *texture.format())
            });
            (render_formats, depth_format)
        };
        let mut sample_count = 0;
        let mips_count = 1;
        if let Some(pipeline) = &self.pipeline {
            binding_data.set_bind_group_layout();
            let mut pipeline = pipeline.get_mut();
            pipeline.init(
                render_context,
                render_formats.iter().collect(),
                depth_format.as_ref(),
                binding_data,
                vertex_layout,
                instance_layout,
            );
            sample_count = if has_multisampling_support() {
                pipeline.data().sampling_count
            } else {
                1
            };
        }
        if self.render_textures.is_empty() && sample_count > 1 {
            self.create_render_target(RenderTarget::Texture {
                width: render_context.webgpu.config.read().unwrap().width,
                height: render_context.webgpu.config.read().unwrap().height,
                format: render_context.webgpu.config.read().unwrap().format.into(),
                sample_count,
                mips_count,
            });
        }
    }
    pub fn color_operations(&self) -> wgpu::Operations<wgpu::Color> {
        wgpu::Operations {
            load: match &self.load_color {
                LoadOperation::Load => wgpu::LoadOp::Load,
                _ => wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            },
            store: match &self.store_color {
                StoreOperation::Store => wgpu::StoreOp::Store,
                _ => wgpu::StoreOp::Discard,
            },
        }
    }

    pub fn depth_operations(&self) -> wgpu::Operations<f32> {
        wgpu::Operations {
            load: match &self.load_depth {
                LoadOperation::Load => wgpu::LoadOp::Load,
                _ => wgpu::LoadOp::Clear(1.),
            },
            store: match &self.store_depth {
                StoreOperation::Store => wgpu::StoreOp::Store,
                _ => wgpu::StoreOp::Discard,
            },
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

        if self.render_textures.is_empty() {
            render_targets_views.push(render_pass_begin_data.surface_view.as_ref());
        } else {
            self.render_textures.iter().for_each(|target| {
                if let Some(texture) = render_pass_begin_data
                    .render_targets
                    .iter()
                    .find(|t| t.id() == target.id())
                {
                    render_targets_views.push(texture.view(0).as_ref());
                }
            });
        }
        if let Some(texture) = self.depth_texture() {
            if let Some(texture) = render_pass_begin_data
                .render_targets
                .iter()
                .find(|t| t.id() == texture.id())
            {
                depth_target_view = Some(texture.view(0).as_ref());
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
                                depth_slice: None,
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
                    ..Default::default()
                })
        };
        {
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass,
                &render_pass_begin_data.render_core_context.device,
                "render_pass::set_pipeline",
            );
            render_pass.set_pipeline(pipeline.render_pipeline());
        }
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
                    render_pass.set_bind_group(index as _, Some(bind_group), &[]);
                });
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

    #[allow(unused_variables)]
    pub fn draw_indexed(
        &self,
        render_context: &RenderContext,
        mut render_pass: wgpu::RenderPass,
        instances: &[DrawIndexedCommand],
    ) {
        inox_profiler::scoped_profile!("render_pass::draw");
        instances.iter().enumerate().for_each(|(i, instance)| {
            inox_profiler::scoped_profile!("render_pass::draw_indexed");
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass,
                &render_context.webgpu.device,
                "render_pass::draw_indexed",
            );
            render_pass.draw_indexed(
                instance.vertex_offset as u32
                    ..(instance.vertex_offset as u32 + instance.vertex_count),
                instance.base_index as _,
                i as _..(i + 1) as _,
            );
        });
    }

    #[allow(unused_mut)]
    pub fn indirect_draw(
        &self,
        render_context: &RenderContext,
        commands: &GPUVector<DrawIndexedCommand>,
        mut render_pass: wgpu::RenderPass,
    ) {
        inox_profiler::scoped_profile!("render_pass::indirect_draw");

        inox_profiler::gpu_scoped_profile!(
            &mut render_pass,
            &render_context.webgpu.device,
            "render_pass::multi_draw_indexed_indirect_count",
        );
        let mut render_pass = render_pass;
        let (commands_id, count) = {
            let commands_guard = commands.read().unwrap();
            (commands_guard.buffer_id(), commands_guard.len() as u32)
        };
        let buffers = render_context.buffers();
        let indirect_buffer = buffers.get(&commands_id).unwrap();
        let gpu_buffer = indirect_buffer.gpu_buffer().unwrap();
        if render_context.capabilities().multi_draw_indirect_count {
            render_pass.multi_draw_indexed_indirect_count(
                gpu_buffer,
                DRAW_INDEXED_COMMAND_OFFSET,
                gpu_buffer,
                0,
                count,
            );
        } else {
            render_pass.multi_draw_indexed_indirect(gpu_buffer, DRAW_INDEXED_COMMAND_OFFSET, count);
        }
    }

    pub fn draw_meshlets(&self, render_context: &RenderContext, mut render_pass: wgpu::RenderPass) {
        inox_profiler::scoped_profile!("render_pass::draw_meshlets");

        let mesh_flags = self.pipeline().get().data().mesh_flags;
        let meshes = render_context.global_buffers().buffer::<GPUMesh>();
        let meshlets = render_context.global_buffers().buffer::<GPUMeshlet>();
        let commands = {
            let mut scratch = self.meshlet_commands.lock().unwrap();
            scratch.clear();
            let meshes = meshes.read().unwrap();
            let meshlets = meshlets.read().unwrap();
            meshes.for_each_data(|_i, mesh_id, mesh| {
                if let Some(meshlets) = meshlets.get(mesh_id) {
                    let flags = (mesh.flags_and_vertices_attribute_layout & 0xFFFF0000) >> 16;
                    if MeshFlags::from(flags) == mesh_flags {
                        for (meshlet_offset, meshlet) in meshlets.iter().enumerate() {
                            let meshlet_index = mesh.meshlets_offset + meshlet_offset as u32;
                            scratch.push(DrawIndexedCommand {
                                vertex_count: meshlet.indices_count,
                                instance_count: 1,
                                base_index: meshlet.indices_offset,
                                vertex_offset: mesh.vertices_position_offset as i32,
                                base_instance: meshlet_index,
                            });
                        }
                    }
                }
            });
            std::mem::take(&mut *scratch)
        };
        for command in commands.iter() {
            inox_profiler::scoped_profile!("render_pass::draw_indexed");
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass,
                &render_context.webgpu.device,
                "render_pass::draw_indexed",
            );
            render_pass.draw_indexed(
                command.base_index..(command.base_index + command.vertex_count),
                command.vertex_offset,
                command.base_instance..(command.base_instance + command.instance_count),
            );
        }
        *self.meshlet_commands.lock().unwrap() = commands;
    }

    #[allow(unused_variables, unused_mut)]
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
            &render_context.webgpu.device,
            "render_pass::draw",
        );
        render_pass.draw(vertices, instances);
    }

    pub fn draw_meshes(&self, render_context: &RenderContext, mut render_pass: wgpu::RenderPass) {
        inox_profiler::scoped_profile!("render_pass::draw_meshes");

        let mesh_flags = self.pipeline().get().data().mesh_flags;
        let meshlets = render_context.global_buffers().buffer::<GPUMeshlet>();
        let meshes = render_context.global_buffers().buffer::<GPUMesh>();
        let commands = {
            let mut scratch = self.meshlet_commands.lock().unwrap();
            scratch.clear();
            let meshlets = meshlets.read().unwrap();
            let meshes = meshes.read().unwrap();
            meshes.for_each_data(|index, mesh_id, mesh| {
                let flags = (mesh.flags_and_vertices_attribute_layout & 0xFFFF0000) >> 16;
                if MeshFlags::from(flags) == mesh_flags {
                    if let Some(meshlets) = meshlets.get(mesh_id) {
                        if let Some(first) = meshlets.first() {
                            let end = meshlets
                                .iter()
                                .map(|meshlet| meshlet.indices_count)
                                .fold(first.indices_offset, |end, count| end + count);
                            scratch.push(DrawIndexedCommand {
                                vertex_count: end - first.indices_offset,
                                instance_count: 1,
                                base_index: first.indices_offset,
                                vertex_offset: mesh.vertices_position_offset as i32,
                                base_instance: index as u32,
                            });
                        }
                    }
                }
            });
            std::mem::take(&mut *scratch)
        };
        for command in commands.iter() {
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass,
                &render_context.webgpu.device,
                "render_pass::draw_indexed",
            );
            render_pass.draw_indexed(
                command.base_index..(command.base_index + command.vertex_count),
                command.vertex_offset,
                command.base_instance..(command.base_instance + command.instance_count),
            );
        }
        *self.meshlet_commands.lock().unwrap() = commands;
    }
}
