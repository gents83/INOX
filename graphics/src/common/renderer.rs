use std::sync::{Arc, RwLock};

use crate::{MaterialInstance, Pipeline, PipelineInstance, TextureInstance};
use nrg_math::*;
use nrg_platform::*;
use nrg_resources::{ResourceRc, ResourceTrait};

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
    pipelines: Vec<Pipeline>,
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
            instance,
            device,
            viewport: Viewport::default(),
            scissors: Scissors::default(),
            texture_handler,
            pipelines: Vec::new(),
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
        pipelines: &mut [ResourceRc<PipelineInstance>],
        materials: &mut [ResourceRc<MaterialInstance>],
        textures: &mut [ResourceRc<TextureInstance>],
    ) -> &mut Self {
        nrg_profiler::scoped_profile!("renderer::prepare_frame");
        self.load_pipelines(pipelines);
        self.load_textures(textures);

        self.prepare_pipelines();
        self.prepare_materials(pipelines, materials);
        self
    }

    pub fn get_texture_handler(&self) -> &TextureHandler {
        &self.texture_handler
    }
    pub fn get_pipelines(&mut self) -> &mut Vec<Pipeline> {
        &mut self.pipelines
    }
    pub fn end_preparation(&mut self) {
        self.state = RendererState::Prepared;
    }

    fn begin_frame(&mut self) -> bool {
        nrg_profiler::scoped_profile!("renderer::begin_frame");

        self.device.begin_frame()
    }

    fn end_frame(&mut self) -> bool {
        nrg_profiler::scoped_profile!("renderer::end_frame");

        self.device.end_frame();
        self.device.submit()
    }

    pub fn draw(&mut self) {
        let mut success = self.begin_frame();
        if !success {
            self.recreate();
        } else {
            nrg_profiler::scoped_profile!("renderer::draw");

            for (pipeline_index, pipeline) in self.pipelines.iter_mut().enumerate() {
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
                        pipeline.update_constant_data();
                        pipeline.update_uniform_buffer([0., 0., 800.].into());
                        pipeline.update_descriptor_sets(self.texture_handler.get_textures());
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

            success = self.end_frame();
        }
        if !success {
            self.recreate();
        }

        self.state = RendererState::Submitted;
    }

    pub fn recreate(&mut self) {
        nrg_profiler::scoped_profile!("renderer::recreate");
        self.device.recreate_swap_chain();

        let device = &self.device;
        self.pipelines.iter_mut().for_each(|p| {
            p.recreate(device);
        });
    }
}

impl Renderer {
    fn load_pipelines(&mut self, pipelines: &mut [ResourceRc<PipelineInstance>]) {
        nrg_profiler::scoped_profile!("renderer::load_pipelines");
        pipelines.iter_mut().for_each(|pipeline_instance| {
            let mut create_pipeline = false;
            if let Some(index) = self
                .pipelines
                .iter()
                .position(|p| p.id() == pipeline_instance.id())
            {
                if !pipeline_instance.get().is_initialized() {
                    //pipeline needs to be recreated
                    let mut pipeline = self.pipelines.remove(index);
                    pipeline.destroy();
                    create_pipeline = true;
                }
            } else {
                create_pipeline = true;
            }
            if create_pipeline {
                let device = &mut self.device;
                self.pipelines.push(Pipeline::create(
                    device,
                    pipeline_instance.id(),
                    pipeline_instance.get().get_data(),
                ));
                pipeline_instance.get_mut().init();
            }
        });
    }

    fn load_textures(&mut self, textures: &mut [ResourceRc<TextureInstance>]) {
        nrg_profiler::scoped_profile!("renderer::load_textures");
        let texture_handler = &mut self.texture_handler;
        textures.iter_mut().for_each(|texture_instance| {
            if texture_instance.get().get_texture_handler_index() == INVALID_INDEX {
                let path = texture_instance.get().get_path().to_path_buf();
                let (handler_index, texture_index, layer_index) =
                    texture_handler.add(path.as_path());
                texture_instance.get_mut().set_texture_data(
                    handler_index,
                    texture_index,
                    layer_index,
                );
            }
        });
    }

    fn prepare_pipelines(&mut self) {
        nrg_profiler::scoped_profile!("renderer::prepare_pipelines");
        self.pipelines
            .sort_by(|a, b| a.get_data().data.index.cmp(&b.get_data().data.index));
        self.pipelines.iter_mut().for_each(|pipeline| {
            pipeline.prepare();
        });
    }

    fn prepare_materials(
        &mut self,
        pipelines: &[ResourceRc<PipelineInstance>],
        materials: &mut [ResourceRc<MaterialInstance>],
    ) {
        nrg_profiler::scoped_profile!("renderer::prepare_materials");
        materials.sort_by(|a, b| {
            let pipeline_a = pipelines
                .iter()
                .find(|&p| p.id() == a.get().get_pipeline_id())
                .unwrap();
            let pipeline_b = pipelines
                .iter()
                .find(|&p| p.id() == b.get().get_pipeline_id())
                .unwrap();
            pipeline_a
                .get()
                .get_data()
                .data
                .index
                .cmp(&pipeline_b.get().get_data().data.index)
        });
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.device.destroy();
        self.instance.destroy();
    }
}
