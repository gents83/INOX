use crate::{FontRc, MaterialRc, PipelineId, PipelineRc, RenderPass, RenderPassRc, TextureRc};
use crate::{Pipeline, RenderPassId};
use nrg_math::*;
use nrg_platform::*;
use nrg_resources::{FileResource, DATA_FOLDER};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use super::device::*;
use super::instance::*;
use super::texture::*;
use super::viewport::*;

pub const INVALID_INDEX: i32 = -1;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum RendererState {
    Init,
    Prepared,
    Submitted,
}

pub struct Renderer {
    pub instance: Instance,
    pub device: Device,
    viewport: Viewport,
    scissors: Scissors,
    texture_handler: TextureHandler,
    state: RendererState,
    render_passes: Vec<RenderPass>,
    pipelines: HashMap<RenderPassId, Vec<Pipeline>>,
}
pub type RendererRw = Arc<RwLock<Renderer>>;

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}

impl Renderer {
    pub fn new(handle: &Handle, enable_debug: bool) -> Self {
        let instance = Instance::create(handle, enable_debug);
        let device = Device::create(&instance);
        let texture_handler = TextureHandler::create(&device);
        Renderer {
            render_passes: Vec::new(),
            pipelines: HashMap::new(),
            instance,
            device,
            viewport: Viewport::default(),
            scissors: Scissors::default(),
            texture_handler,
            state: RendererState::Init,
        }
    }

    pub fn state(&self) -> RendererState {
        self.state
    }

    pub fn set_viewport_size(&mut self, size: Vector2) -> &mut Self {
        self.viewport.width = size.x as _;
        self.viewport.height = size.y as _;
        self.scissors.width = self.viewport.width;
        self.scissors.height = self.viewport.height;
        self
    }

    pub fn get_viewport_size(&self) -> Vector2 {
        Vector2::new(self.viewport.width, self.viewport.height)
    }

    pub fn prepare_frame(
        &mut self,
        render_passes: &mut [RenderPassRc],
        pipelines: &mut [PipelineRc],
        materials: &mut [MaterialRc],
        textures: &mut [TextureRc],
        fonts: &[FontRc],
    ) -> &mut Self {
        nrg_profiler::scoped_profile!("renderer::prepare_frame");
        self.load_render_passes(render_passes);
        self.load_pipelines(pipelines, render_passes);
        self.load_textures(textures, fonts);

        self.prepare_pipelines();
        self.prepare_materials(pipelines, materials);
        self
    }

    pub fn get_texture_handler(&self) -> &TextureHandler {
        &self.texture_handler
    }
    pub fn get_texture_handler_mut(&mut self) -> &mut TextureHandler {
        &mut self.texture_handler
    }
    pub fn get_pipelines_with_id(&mut self, pipeline_id: PipelineId) -> Vec<&mut Pipeline> {
        let mut pipelines = Vec::new();
        for (_id, pipelines_in_pass) in self.pipelines.iter_mut() {
            for p in pipelines_in_pass.iter_mut() {
                if p.id() == pipeline_id {
                    pipelines.push(p);
                }
            }
        }
        pipelines
    }
    pub fn end_preparation(&mut self) {
        self.state = RendererState::Prepared;
    }

    fn begin_frame(&mut self) -> bool {
        nrg_profiler::scoped_profile!("renderer::begin_frame");

        self.device.begin_frame()
    }

    fn end_frame(&mut self) {
        nrg_profiler::scoped_profile!("renderer::end_frame");

        self.device.end_frame();
        self.device.submit();
    }
    fn present(&mut self) -> bool {
        nrg_profiler::scoped_profile!("renderer::present");
        self.device.present()
    }

    pub fn draw(&mut self, view: &Matrix4, proj: &Matrix4) {
        if self.state == RendererState::Submitted {
            return;
        }

        let mut success = self.begin_frame();
        if success {
            nrg_profiler::scoped_profile!("renderer::draw");

            for (render_pass_index, render_pass) in self.render_passes.iter_mut().enumerate() {
                nrg_profiler::scoped_profile!(format!(
                    "renderer::render_pass[{}]",
                    render_pass_index
                )
                .as_str());

                render_pass.begin();

                for (render_pass_id, pipelines) in self.pipelines.iter_mut() {
                    if *render_pass_id != render_pass.id() {
                        continue;
                    }
                    for (pipeline_index, pipeline) in pipelines.iter_mut().enumerate() {
                        if !pipeline.is_empty() {
                            nrg_profiler::scoped_profile!(format!(
                                "renderer::draw_pipeline[{}]",
                                pipeline_index
                            )
                            .as_str());

                            {
                                nrg_profiler::scoped_profile!(format!(
                                    "renderer::update_uniforms_and_descriptors[{}]",
                                    pipeline_index
                                )
                                .as_str());
                                pipeline.update_runtime_data(view, proj);
                                pipeline.update_descriptor_sets(
                                    self.texture_handler.get_textures_atlas(),
                                );
                            }

                            {
                                nrg_profiler::scoped_profile!(format!(
                                    "renderer::draw_pipeline_begin[{}]",
                                    pipeline_index
                                )
                                .as_str());
                                pipeline.begin();
                            }

                            {
                                nrg_profiler::scoped_profile!(format!(
                                    "renderer::draw_pipeline_call[{}]",
                                    pipeline_index
                                )
                                .as_str());
                                {
                                    nrg_profiler::scoped_profile!(format!(
                                        "renderer::draw_pipeline_call[{}]_bind_vertices",
                                        pipeline_index
                                    )
                                    .as_str());
                                    pipeline.bind_vertices();
                                }
                                {
                                    nrg_profiler::scoped_profile!(format!(
                                        "renderer::draw_pipeline_call[{}]_bind_indirect",
                                        pipeline_index
                                    )
                                    .as_str());
                                    pipeline.bind_indirect();
                                }
                                {
                                    nrg_profiler::scoped_profile!(format!(
                                        "renderer::draw_pipeline_call[{}]_bind_indices",
                                        pipeline_index
                                    )
                                    .as_str());
                                    pipeline.bind_indices();
                                }
                                {
                                    nrg_profiler::scoped_profile!(format!(
                                        "renderer::draw_pipeline_call[{}]_draw_indirect",
                                        pipeline_index
                                    )
                                    .as_str());
                                    pipeline.draw_indirect();
                                }
                            }

                            {
                                nrg_profiler::scoped_profile!(format!(
                                    "renderer::draw_pipeline_end[{}]",
                                    pipeline_index
                                )
                                .as_str());
                                pipeline.end();
                            }
                        }
                    }
                }

                render_pass.end();
            }

            self.end_frame();
            success = self.present();
        }
        if !success {
            self.recreate();
        }
        self.state = RendererState::Submitted;
    }

    pub fn recreate(&mut self) {
        nrg_profiler::scoped_profile!("renderer::recreate");
        self.device.recreate_swap_chain();
        self.pipelines.iter_mut().for_each(|(_id, pipelines)| {
            pipelines.iter_mut().for_each(|p| p.destroy());
        });
        self.pipelines.clear();
        self.render_passes.iter_mut().for_each(|r| r.destroy());
        self.render_passes.clear();
    }
}

impl Renderer {
    fn load_render_passes(&mut self, render_passes: &mut [RenderPassRc]) {
        nrg_profiler::scoped_profile!("renderer::load_render_passes");
        render_passes.iter_mut().for_each(|render_pass_instance| {
            let mut should_create = false;
            if let Some(index) = self
                .render_passes
                .iter()
                .position(|r| r.id() == render_pass_instance.id())
            {
                if !render_pass_instance.resource().get().is_initialized() {
                    //render pass needs to be recreated
                    let mut render_pass = self.render_passes.remove(index);
                    render_pass.destroy();
                    should_create = true;
                }
            } else {
                should_create = true;
            }
            if should_create {
                let device = &mut self.device;
                if render_pass_instance
                    .resource()
                    .get()
                    .data()
                    .render_to_texture
                {
                    if let Some(texture) = render_pass_instance.resource().get().color_texture() {
                        if self
                            .texture_handler
                            .get_texture_atlas(texture.id())
                            .is_none()
                        {
                            self.texture_handler.add_render_target(
                                device,
                                texture.id(),
                                texture.resource().get().width(),
                                texture.resource().get().height(),
                                false,
                            );
                        }
                    }
                    if let Some(texture) = render_pass_instance.resource().get().depth_texture() {
                        if self
                            .texture_handler
                            .get_texture_atlas(texture.id())
                            .is_none()
                        {
                            self.texture_handler.add_render_target(
                                device,
                                texture.id(),
                                texture.resource().get().width(),
                                texture.resource().get().height(),
                                true,
                            );
                        }
                    }
                    let color_texture = if let Some(texture) =
                        render_pass_instance.resource().get().color_texture()
                    {
                        self.texture_handler
                            .get_texture_atlas(texture.id())
                            .map(|texture_atlas| texture_atlas.get_texture())
                    } else {
                        None
                    };
                    let depth_texture = if let Some(texture) =
                        render_pass_instance.resource().get().depth_texture()
                    {
                        self.texture_handler
                            .get_texture_atlas(texture.id())
                            .map(|texture_atlas| texture_atlas.get_texture())
                    } else {
                        None
                    };
                    self.render_passes
                        .push(RenderPass::create_with_render_target(
                            device,
                            render_pass_instance.id(),
                            render_pass_instance.resource().get().data(),
                            color_texture,
                            depth_texture,
                        ));
                } else {
                    self.render_passes.push(RenderPass::create_default(
                        device,
                        render_pass_instance.id(),
                        render_pass_instance.resource().get().data(),
                    ));
                }
                render_pass_instance.resource().get_mut().init();
            }
        });
    }
    fn load_pipelines(&mut self, pipelines: &mut [PipelineRc], render_passes: &mut [RenderPassRc]) {
        nrg_profiler::scoped_profile!("renderer::load_pipelines");
        pipelines.iter_mut().for_each(|pipeline_instance| {
            let mut create_pipeline = true;

            self.pipelines.iter_mut().for_each(|(_id, pipelines)| {
                if let Some(index) = pipelines
                    .iter_mut()
                    .position(|p| p.id() == pipeline_instance.id())
                {
                    if pipeline_instance.resource().get().is_initialized() {
                        create_pipeline = false;
                    } else {
                        //pipeline needs to be recreated
                        let mut pipeline = pipelines.remove(index);
                        pipeline.destroy();
                        create_pipeline = true;
                    }
                }
            });

            if create_pipeline {
                render_passes.iter().for_each(|render_pass| {
                    if pipeline_instance
                        .resource()
                        .get()
                        .should_draw_in_render_pass(&render_pass.resource().get().data().name)
                    {
                        let device = &mut self.device;
                        let pipelines = self
                            .pipelines
                            .entry(render_pass.id())
                            .or_insert_with(Vec::new);
                        pipelines.push(Pipeline::create(
                            device,
                            pipeline_instance.id(),
                            pipeline_instance.resource().get().data(),
                            self.render_passes.first().unwrap(),
                        ));
                        pipeline_instance.resource().get_mut().init();
                    }
                });
            }
        });
    }

    fn load_textures(&mut self, textures: &mut [TextureRc], fonts: &[FontRc]) {
        nrg_profiler::scoped_profile!("renderer::load_textures");
        let texture_handler = &mut self.texture_handler;
        textures.iter_mut().for_each(|texture_instance| {
            if !texture_instance.resource().get().is_initialized() {
                if texture_instance.resource().get().texture_index() != INVALID_INDEX {
                    //texture needs to be recreated
                    texture_handler.remove(texture_instance.id());
                }
                let path = convert_from_local_path(
                    PathBuf::from(DATA_FOLDER).as_path(),
                    texture_instance.resource().get().path(),
                );
                if let Some(texture_info) = texture_handler.get_texture_info(texture_instance.id())
                {
                    texture_instance
                        .resource()
                        .get_mut()
                        .set_texture_info(texture_info);
                } else {
                    let texture_info = if let Some(image_data) =
                        texture_instance.resource().get_mut().image_data()
                    {
                        texture_handler.add(texture_instance.id(), image_data)
                    } else if is_texture(path.as_path()) {
                        texture_handler.add_from_path(texture_instance.id(), path.as_path())
                    } else if let Some(font) =
                        fonts.iter().find(|f| f.resource().get().path() == path)
                    {
                        texture_handler.add(
                            texture_instance.id(),
                            font.resource().get().font().get_texture(),
                        )
                    } else {
                        panic!("Unable to load texture with path {:?}", path.as_path());
                    };
                    texture_instance
                        .resource()
                        .get_mut()
                        .set_texture_info(&texture_info);
                }
            }
        });
    }

    fn prepare_pipelines(&mut self) {
        nrg_profiler::scoped_profile!("renderer::prepare_pipelines");
        self.pipelines.iter_mut().for_each(|(_id, pipelines)| {
            pipelines.iter_mut().for_each(|pipeline| {
                pipeline.prepare();
            });
        });
    }

    fn prepare_materials(&mut self, pipelines: &[PipelineRc], materials: &mut [MaterialRc]) {
        nrg_profiler::scoped_profile!("renderer::prepare_materials");
        materials.sort_by(|a, b| {
            let pipeline_a = pipelines
                .iter()
                .position(|p| p.id() == a.resource().get().pipeline().id())
                .unwrap();
            let pipeline_b = pipelines
                .iter()
                .position(|p| p.id() == b.resource().get().pipeline().id())
                .unwrap();
            pipeline_a.cmp(&pipeline_b)
        });
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.device.destroy();
        self.instance.destroy();
    }
}
