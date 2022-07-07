use std::path::PathBuf;

use crate::{
    AsBinding, BindingData, BindingInfo, CommandBuffer, ComputePass, ComputePassData, GpuBuffer,
    MeshFlags, Pass, RenderContext, RenderCoreContext, ShaderStage, Texture, TextureFormat,
    TextureId, TextureUsage, DEFAULT_HEIGHT, DEFAULT_WIDTH,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Handle, Resource};
use inox_uid::generate_random_uid;

pub const PBR_PIPELINE: &str = "pipelines/Pbr.compute_pipeline";
pub const PBR_PASS_NAME: &str = "PbrPass";
pub const PBR_TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

#[derive(Default)]
struct PbrPassData {
    dimensions: [u32; 2],
    albedo_texture_index: u32,
    normals_texture_index: u32,
    material_params_texture_index: u32,
    _padding: [u32; 3],
}

impl AsBinding for PbrPassData {
    fn is_dirty(&self) -> bool {
        true
    }
    fn set_dirty(&mut self, _is_dirty: bool) {}
    fn size(&self) -> u64 {
        std::mem::size_of_val(&self.dimensions) as u64
            + std::mem::size_of_val(&self.albedo_texture_index) as u64
            + std::mem::size_of_val(&self.normals_texture_index) as u64
            + std::mem::size_of_val(&self.material_params_texture_index) as u64
            + std::mem::size_of_val(&self._padding) as u64
    }

    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut GpuBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, &[self.dimensions]);
        buffer.add_to_gpu_buffer(render_core_context, &[self.albedo_texture_index]);
        buffer.add_to_gpu_buffer(render_core_context, &[self.normals_texture_index]);
        buffer.add_to_gpu_buffer(render_core_context, &[self.material_params_texture_index]);
        buffer.add_to_gpu_buffer(render_core_context, &[self._padding]);
    }
}
pub struct PbrPass {
    context: ContextRc,
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    data: PbrPassData,
    textures: Vec<TextureId>,
    render_target: Handle<Texture>,
}
unsafe impl Send for PbrPass {}
unsafe impl Sync for PbrPass {}

impl Pass for PbrPass {
    fn name(&self) -> &str {
        PBR_PASS_NAME
    }
    fn static_name() -> &'static str {
        PBR_PASS_NAME
    }
    fn create(context: &ContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: PBR_PASS_NAME.to_string(),
            pipelines: vec![PathBuf::from(PBR_PIPELINE)],
        };
        Self {
            context: context.clone(),
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                data,
                None,
            ),
            binding_data: BindingData::default(),
            textures: Vec::new(),
            data: PbrPassData {
                dimensions: [DEFAULT_WIDTH, DEFAULT_HEIGHT],
                ..Default::default()
            },
            render_target: None,
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        inox_profiler::scoped_profile!("default_pass::init");

        if self.render_target.is_none()
            && (self.data.dimensions[0] != render_context.core.config.width
                || self.data.dimensions[1] != render_context.core.config.height)
        {
            self.data.dimensions = [
                render_context.core.config.width,
                render_context.core.config.height,
            ];
            self.data.set_dirty(true);
        }

        let mesh_flags = MeshFlags::Visible | MeshFlags::Opaque;

        if !render_context.has_instances(mesh_flags)
            || render_context.render_buffers.materials.is_empty()
            || render_context.render_buffers.textures.is_empty()
            || self.textures.len() < 3
        {
            return;
        }

        if let Some(albedo) = render_context
            .render_buffers
            .textures
            .get(&self.textures[0])
        {
            if self.data.albedo_texture_index != albedo.get_texture_index() {
                self.data.albedo_texture_index = albedo.get_texture_index();
                self.data.set_dirty(true);
            }
        }
        if let Some(normals) = render_context
            .render_buffers
            .textures
            .get(&self.textures[1])
        {
            if self.data.normals_texture_index != normals.get_texture_index() {
                self.data.normals_texture_index = normals.get_texture_index();
                self.data.set_dirty(true);
            }
        }
        if let Some(materials) = render_context
            .render_buffers
            .textures
            .get(&self.textures[2])
        {
            if self.data.material_params_texture_index != materials.get_texture_index() {
                self.data.material_params_texture_index = materials.get_texture_index();
                self.data.set_dirty(true);
            }
        }

        self.binding_data
            .add_uniform_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.constant_data,
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_uniform_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut self.data,
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.materials,
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.textures,
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_textures(
                render_context,
                if self.render_target.is_some() {
                    vec![self.render_target.as_ref().unwrap()]
                } else {
                    Vec::new()
                },
                BindingInfo {
                    group_index: 1,
                    stage: ShaderStage::Compute,
                    read_only: false,
                    ..Default::default()
                },
            )
            .add_textures(
                &render_context.texture_handler,
                if self.render_target.is_some() {
                    vec![self.render_target.as_ref().unwrap().id()]
                } else {
                    Vec::new()
                },
                None,
                BindingInfo {
                    group_index: 2,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            );
        self.binding_data.send_to_gpu(render_context, PBR_PASS_NAME);

        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &self.binding_data);
    }

    fn update(&mut self, _render_context: &mut RenderContext, command_buffer: &mut CommandBuffer) {
        let pass = self.compute_pass.get();

        let compute_pass = pass.begin(&self.binding_data, command_buffer);
        let max_cluster_size = 16;
        self.data.dimensions = [16, 16];
        let width = max_cluster_size
            * ((self.data.dimensions[0] + max_cluster_size - 1) / max_cluster_size);
        let height = max_cluster_size
            * ((self.data.dimensions[1] + max_cluster_size - 1) / max_cluster_size);
        pass.dispatch(compute_pass, width, height, 1);
    }
}

impl PbrPass {
    pub fn add_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.textures.push(*texture_id);
        self
    }
    pub fn resolution(&mut self, width: u32, height: u32) -> &mut Self {
        self.render_target = Some(Texture::create_from_format(
            self.context.shared_data(),
            self.context.message_hub(),
            width,
            height,
            PBR_TEXTURE_FORMAT,
            TextureUsage::TextureBinding
                | TextureUsage::CopySrc
                | TextureUsage::CopyDst
                | TextureUsage::RenderAttachment
                | TextureUsage::StorageBinding,
        ));
        self.data.dimensions = [width, height];
        self.data.set_dirty(true);
        self
    }
    pub fn render_target_id(&self) -> &TextureId {
        self.render_target.as_ref().unwrap().id()
    }
}
