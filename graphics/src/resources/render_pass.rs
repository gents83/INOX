use nrg_resources::{
    DataTypeResource, Handle, Resource, ResourceData, ResourceId, SharedData, SharedDataRw,
};
use nrg_serialize::{generate_uid_from_string, Uid};

use crate::{
    api::backend::{self, BackendPhysicalDevice},
    CommandBuffer, Device, MeshCategoryId, Pipeline, RenderPassData, RenderTarget, Texture,
    TextureHandler,
};

pub type RenderPassId = Uid;

#[derive(Default)]
pub struct RenderPass {
    id: ResourceId,
    backend_render_pass: Option<backend::BackendRenderPass>,
    data: RenderPassData,
    color_texture: Handle<Texture>,
    depth_texture: Handle<Texture>,
    pipeline: Handle<Pipeline>,
    is_initialized: bool,
    mesh_category_to_draw: Vec<MeshCategoryId>,
    texture_to_recycle: Vec<Resource<Texture>>,
    command_buffer: Option<CommandBuffer>,
}

impl std::ops::Deref for RenderPass {
    type Target = backend::BackendRenderPass;
    fn deref(&self) -> &Self::Target {
        self.backend_render_pass.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for RenderPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.backend_render_pass.as_mut().unwrap()
    }
}

impl ResourceData for RenderPass {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl DataTypeResource for RenderPass {
    type DataType = RenderPassData;
    fn create_from_data(
        shared_data: &SharedDataRw,
        render_pass_data: Self::DataType,
    ) -> Resource<Self> {
        if let Some(render_pass) =
            RenderPass::find_from_name(shared_data, render_pass_data.name.as_str())
        {
            return render_pass;
        }

        let pipeline = render_pass_data
            .pipeline
            .as_ref()
            .map(|pipeline_data| Pipeline::create_from_data(shared_data, pipeline_data.clone()));

        let mut mesh_category_to_draw = Vec::new();
        render_pass_data
            .mesh_category_to_draw
            .iter()
            .for_each(|name| {
                mesh_category_to_draw.push(MeshCategoryId::new(name.as_str()));
            });

        SharedData::add_resource(
            shared_data,
            RenderPass {
                id: generate_uid_from_string(render_pass_data.name.as_str()),
                data: render_pass_data.clone(),
                pipeline,
                mesh_category_to_draw,
                ..Default::default()
            },
        )
    }
}

impl RenderPass {
    pub fn find_from_name(shared_data: &SharedDataRw, render_pass_name: &str) -> Handle<Self> {
        SharedData::match_resource(shared_data, |r: &RenderPass| {
            r.data.name == render_pass_name
        })
    }
    pub fn data(&self) -> &RenderPassData {
        &self.data
    }
    pub fn pipeline(&self) -> &Handle<Pipeline> {
        &self.pipeline
    }
    pub fn mesh_category_to_draw(&self) -> &[MeshCategoryId] {
        &self.mesh_category_to_draw
    }

    pub fn add_category_to_draw(&mut self, mesh_category_id: MeshCategoryId) -> &mut Self {
        self.mesh_category_to_draw.push(mesh_category_id);
        self
    }

    fn add_texture_as_render_target<'a>(
        device: &Device,
        physical_device: &BackendPhysicalDevice,
        texture: &Handle<Texture>,
        texture_handler: &'a mut TextureHandler,
        should_update_from_gpu: bool,
    ) {
        if let Some(t) = texture {
            if texture_handler.get_texture_atlas(t.id()).is_none() {
                //println!("Adding texture {:?}", t.id());
                t.get_mut().set_update_from_gpu(should_update_from_gpu);
                texture_handler.add_render_target(
                    device,
                    physical_device,
                    t.id(),
                    t.get().width(),
                    t.get().height(),
                    false,
                );
            }
        }
    }
    fn get_real_texture<'a>(
        texture: &Handle<Texture>,
        texture_handler: &'a TextureHandler,
    ) -> Option<&'a backend::BackendTexture> {
        let mut real_texture = None;
        if let Some(t) = texture {
            real_texture = texture_handler
                .get_texture_atlas(t.id())
                .map(|texture_atlas| texture_atlas.get_texture())
        }
        real_texture
    }
    fn create_default(
        &mut self,
        device: &Device,
        physical_device: &BackendPhysicalDevice,
    ) -> &mut Self {
        self.backend_render_pass = Some(backend::BackendRenderPass::create_default(
            device,
            physical_device,
            &self.data,
            None,
            None,
        ));
        self
    }
    fn create_with_render_target(
        &mut self,
        device: &Device,
        physical_device: &BackendPhysicalDevice,
        texture_handler: &mut TextureHandler,
        should_update_from_gpu: bool,
    ) -> &mut Self {
        Self::add_texture_as_render_target(
            device,
            physical_device,
            &self.color_texture,
            texture_handler,
            should_update_from_gpu,
        );
        Self::add_texture_as_render_target(
            device,
            physical_device,
            &self.depth_texture,
            texture_handler,
            false,
        );
        let color_texture = Self::get_real_texture(&self.color_texture, texture_handler);
        let depth_texture = Self::get_real_texture(&self.depth_texture, texture_handler);

        self.backend_render_pass = Some(backend::BackendRenderPass::create_default(
            device,
            physical_device,
            &self.data,
            color_texture,
            depth_texture,
        ));
        self
    }
    pub fn init(
        &mut self,
        device: &mut Device,
        physical_device: &BackendPhysicalDevice,
        texture_handler: &mut TextureHandler,
    ) -> &mut Self {
        if let Some(backend_render_pass) = &mut self.backend_render_pass {
            backend_render_pass.destroy(device);
        }
        self.texture_to_recycle.iter().for_each(|t| {
            //println!("Removing texture {:?}", t.id());
            texture_handler.remove(device, t.id());
        });
        self.texture_to_recycle.clear();
        if self.data.render_target == RenderTarget::Screen {
            self.create_default(device, physical_device);
        } else {
            self.create_with_render_target(
                device,
                physical_device,
                texture_handler,
                self.data.render_target == RenderTarget::TextureAndReadback,
            );
        }
        if let Some(pipeline) = &self.pipeline {
            pipeline.get_mut().init(device, physical_device, self);
        }
        self.is_initialized = true;
        self
    }
    pub fn color_texture(&self) -> &Handle<Texture> {
        &self.color_texture
    }
    pub fn depth_texture(&self) -> &Handle<Texture> {
        &self.depth_texture
    }
    pub fn set_color_texture(&mut self, color_texture: Resource<Texture>) -> &mut Self {
        if let Some(texture) = &self.color_texture {
            self.texture_to_recycle.push(texture.clone());
        }
        self.color_texture = Some(color_texture);
        self.invalidate();
        self
    }
    pub fn set_depth_texture(&mut self, depth_texture: Resource<Texture>) -> &mut Self {
        if let Some(texture) = &self.depth_texture {
            self.texture_to_recycle.push(texture.clone());
        }
        self.depth_texture = Some(depth_texture);
        self.invalidate();
        self
    }

    pub fn invalidate(&mut self) {
        self.is_initialized = false;
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    pub fn get_framebuffer_width(&self) -> u32 {
        if let Some(backend_render_pass) = &self.backend_render_pass {
            backend_render_pass.get_framebuffer_width()
        } else {
            0
        }
    }

    pub fn get_framebuffer_height(&self) -> u32 {
        if let Some(backend_render_pass) = &self.backend_render_pass {
            backend_render_pass.get_framebuffer_height()
        } else {
            0
        }
    }

    pub fn acquire_command_buffer(&mut self, device: &mut Device) {
        self.command_buffer = Some(CommandBuffer::new(device));
    }
    pub fn get_command_buffer(&self) -> &CommandBuffer {
        self.command_buffer.as_ref().unwrap()
    }

    pub fn draw(&mut self, device: &Device) {
        self.begin(device);
        if let Some(command_buffer) = self.command_buffer.take() {
            command_buffer.execute(device);
        }
        self.end(device);
    }

    pub fn begin_command_buffer(&self, device: &Device) {
        if let Some(backend_render_pass) = &self.backend_render_pass {
            let command_buffer = self.get_command_buffer();
            device.begin_command_buffer(command_buffer, backend_render_pass);
        }
    }

    pub fn end_command_buffer(&self, device: &Device) {
        let command_buffer = self.get_command_buffer();
        device.end_command_buffer(command_buffer);
    }

    fn begin(&self, device: &Device) {
        if let Some(backend_render_pass) = &self.backend_render_pass {
            backend_render_pass.begin(device);
        }
    }

    fn end(&self, device: &Device) {
        if let Some(backend_render_pass) = &self.backend_render_pass {
            backend_render_pass.end(device);
        }
    }
}
