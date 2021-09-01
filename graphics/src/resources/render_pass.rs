use nrg_resources::{
    DataTypeResource, ResourceData, ResourceId, ResourceRef, SharedData, SharedDataRw,
};
use nrg_serialize::{generate_uid_from_string, Uid, INVALID_UID};

use crate::{
    api::backend, Device, MeshCategoryId, Pipeline, PipelineRc, RenderPassData, TextureHandler,
    TextureRc,
};

pub type RenderPassId = Uid;
pub type RenderPassRc = ResourceRef<RenderPass>;

pub struct RenderPass {
    id: ResourceId,
    backend_render_pass: Option<backend::BackendRenderPass>,
    data: RenderPassData,
    color_texture: Option<TextureRc>,
    depth_texture: Option<TextureRc>,
    pipeline: Option<PipelineRc>,
    is_initialized: bool,
    mesh_category_to_draw: Vec<MeshCategoryId>,
    texture_to_recycle: Vec<TextureRc>,
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

impl Default for RenderPass {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            data: RenderPassData::default(),
            color_texture: None,
            depth_texture: None,
            is_initialized: false,
            backend_render_pass: None,
            pipeline: None,
            mesh_category_to_draw: Vec::new(),
            texture_to_recycle: Vec::new(),
        }
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
    ) -> RenderPassRc {
        if let Some(render_pass) =
            RenderPass::find_from_name(shared_data, render_pass_data.name.as_str())
        {
            return render_pass;
        }

        let pipeline = render_pass_data
            .pipeline
            .as_ref()
            .map(|pipeline_data| Pipeline::create_from_data(shared_data, pipeline_data.clone()));
        SharedData::add_resource(
            shared_data,
            RenderPass {
                id: generate_uid_from_string(render_pass_data.name.as_str()),
                data: render_pass_data.clone(),
                pipeline,
                ..Default::default()
            },
        )
    }
}

impl RenderPass {
    pub fn find_from_name(
        shared_data: &SharedDataRw,
        render_pass_name: &str,
    ) -> Option<RenderPassRc> {
        SharedData::match_resource(shared_data, |r: &RenderPass| {
            r.data.name == render_pass_name
        })
    }
    pub fn data(&self) -> &RenderPassData {
        &self.data
    }
    pub fn pipeline(&self) -> &Option<PipelineRc> {
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
        texture: &Option<TextureRc>,
        texture_handler: &'a mut TextureHandler,
    ) {
        if let Some(t) = texture {
            if texture_handler.get_texture_atlas(t.id()).is_none() {
                println!("Adding texture {:?}", t.id());
                texture_handler.add_render_target(
                    device,
                    t.id(),
                    t.resource().get().width(),
                    t.resource().get().height(),
                    false,
                );
            }
        }
    }
    fn get_real_texture<'a>(
        texture: &Option<TextureRc>,
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
    fn create_default(&mut self, device: &Device) -> &mut Self {
        self.backend_render_pass = Some(backend::BackendRenderPass::create_default(
            &*device, &self.data, None, None,
        ));
        self
    }
    fn create_with_render_target(
        &mut self,
        device: &Device,
        texture_handler: &mut TextureHandler,
    ) -> &mut Self {
        Self::add_texture_as_render_target(device, &self.color_texture, texture_handler);
        Self::add_texture_as_render_target(device, &self.depth_texture, texture_handler);
        let color_texture = Self::get_real_texture(&self.color_texture, texture_handler);
        let depth_texture = Self::get_real_texture(&self.depth_texture, texture_handler);

        self.backend_render_pass = Some(backend::BackendRenderPass::create_default(
            &*device,
            &self.data,
            color_texture,
            depth_texture,
        ));
        self
    }
    pub fn init(&mut self, device: &Device, texture_handler: &mut TextureHandler) -> &mut Self {
        if let Some(backend_render_pass) = &mut self.backend_render_pass {
            backend_render_pass.destroy(&*device);
        }
        self.texture_to_recycle.iter().for_each(|t| {
            println!("Removing texture {:?}", t.id());
            texture_handler.remove(device, t.id());
        });
        self.texture_to_recycle.clear();
        if self.data.render_to_texture {
            self.create_with_render_target(device, texture_handler);
        } else {
            self.create_default(device);
        }
        if let Some(pipeline) = &self.pipeline {
            pipeline.resource().get_mut().init(device, self);
        }
        self.is_initialized = true;
        self
    }
    pub fn color_texture(&self) -> Option<TextureRc> {
        self.color_texture.clone()
    }
    pub fn depth_texture(&self) -> Option<TextureRc> {
        self.depth_texture.clone()
    }
    pub fn set_color_texture(&mut self, color_texture: TextureRc) -> &mut Self {
        if let Some(texture) = &self.color_texture {
            self.texture_to_recycle.push(texture.clone());
        }
        self.color_texture = Some(color_texture);
        self.invalidate();
        self
    }
    pub fn set_depth_texture(&mut self, depth_texture: TextureRc) -> &mut Self {
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

    pub fn begin(&self, device: &Device) {
        if let Some(backend_render_pass) = &self.backend_render_pass {
            backend_render_pass.begin(&*device);
        }
    }

    pub fn end(&self, device: &Device) {
        if let Some(backend_render_pass) = &self.backend_render_pass {
            backend_render_pass.end(&*device);
        }
    }
}
