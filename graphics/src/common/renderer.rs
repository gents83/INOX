use std::sync::{Arc, RwLock};

use crate::{MaterialInstance, MeshInstance, PipelineInstance, TextureInstance};
use nrg_math::*;
use nrg_platform::*;
use nrg_resources::ResourceRefMut;

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
}
pub type RendererRw = Arc<RwLock<Renderer>>;

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
            state: RendererState::Init,
        }
    }

    pub fn get_state(&self) -> RendererState {
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
        pipelines: &mut [ResourceRefMut<PipelineInstance>],
        materials: &mut [ResourceRefMut<MaterialInstance>],
        meshes: &mut [ResourceRefMut<MeshInstance>],
        textures: &mut [ResourceRefMut<TextureInstance>],
    ) -> &mut Self {
        nrg_profiler::scoped_profile!("renderer::prepare_frame");
        self.load_pipelines(pipelines);
        self.load_textures(textures);

        self.prepare_pipelines(pipelines);
        self.prepare_materials(pipelines, materials);
        self.prepare_meshes(pipelines, materials, meshes, textures);

        self.state = RendererState::Prepared;
        self
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

    pub fn draw(&mut self, pipelines: &mut [ResourceRefMut<PipelineInstance>]) {
        let mut success = self.begin_frame();
        if !success {
            self.recreate(pipelines);
        } else {
            nrg_profiler::scoped_profile!("renderer::draw");

            for (pipeline_index, pipeline_instance) in pipelines.iter_mut().enumerate() {
                if pipeline_instance.is_initialized() && !pipeline_instance.is_empty() {
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
                        pipeline_instance.update_uniform_buffer([0., 0., 800.].into());
                        pipeline_instance
                            .update_descriptor_sets(self.texture_handler.get_textures());
                    }

                    {
                        nrg_profiler::scoped_profile!(format!(
                            "renderer::draw_pipeline_begin[{}]",
                            pipeline_index
                        )
                        .as_str());
                        pipeline_instance.begin();
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
                            pipeline_instance.bind_vertices();
                        }
                        {
                            nrg_profiler::scoped_profile!(format!(
                                "renderer::draw_pipeline_call[{}]_bind_indirect",
                                pipeline_index
                            )
                            .as_str());
                            pipeline_instance.bind_indirect();
                        }
                        {
                            nrg_profiler::scoped_profile!(format!(
                                "renderer::draw_pipeline_call[{}]_bind_indices",
                                pipeline_index
                            )
                            .as_str());
                            pipeline_instance.bind_indices();
                        }
                        {
                            nrg_profiler::scoped_profile!(format!(
                                "renderer::draw_pipeline_call[{}]_draw_indirect",
                                pipeline_index
                            )
                            .as_str());
                            pipeline_instance.draw_indirect();
                        }
                    }

                    {
                        nrg_profiler::scoped_profile!(format!(
                            "renderer::draw_pipeline_end[{}]",
                            pipeline_index
                        )
                        .as_str());
                        pipeline_instance.end();
                    }
                }
            }

            success = self.end_frame();
        }
        if !success {
            self.recreate(pipelines);
        }

        self.state = RendererState::Submitted;
    }

    pub fn recreate(&mut self, pipelines: &mut [ResourceRefMut<PipelineInstance>]) {
        nrg_profiler::scoped_profile!("renderer::recreate");
        self.device.recreate_swap_chain();

        for pipeline_instance in pipelines.iter_mut() {
            if pipeline_instance.is_initialized() {
                pipeline_instance.recreate(&self.device);
            }
        }
    }
}

impl Renderer {
    fn load_pipelines(&mut self, pipelines: &mut [ResourceRefMut<PipelineInstance>]) {
        nrg_profiler::scoped_profile!("renderer::load_pipelines");
        let device = &mut self.device;
        pipelines.iter_mut().for_each(|pipeline_instance| {
            pipeline_instance.init(&device);
        });
    }

    fn load_textures(&mut self, textures: &mut [ResourceRefMut<TextureInstance>]) {
        nrg_profiler::scoped_profile!("renderer::load_textures");
        let texture_handler = &mut self.texture_handler;
        textures.iter_mut().for_each(|texture_instance| {
            if texture_instance.get_texture_index() == INVALID_INDEX {
                let path = texture_instance.get_path().to_path_buf();
                texture_instance.set_texture_index(texture_handler.add(path.as_path()) as _);
            }
        });
    }

    fn prepare_pipelines(&mut self, pipelines: &mut [ResourceRefMut<PipelineInstance>]) {
        nrg_profiler::scoped_profile!("renderer::prepare_pipelines");
        pipelines.sort_by(|a, b| a.get_data().data.index.cmp(&b.get_data().data.index));

        let mut pipeline_index = 0;
        pipelines.iter_mut().for_each(|pipeline| {
            pipeline.prepare();
            pipeline_index += 1;
        });
    }

    fn prepare_materials(
        &mut self,
        pipelines: &[ResourceRefMut<PipelineInstance>],
        materials: &mut [ResourceRefMut<MaterialInstance>],
    ) {
        nrg_profiler::scoped_profile!("renderer::prepare_materials");
        materials.sort_by(|a, b| {
            let pipeline_a = pipelines
                .iter()
                .find(|&p| p.id() == a.get_pipeline_id())
                .unwrap();
            let pipeline_b = pipelines
                .iter()
                .find(|&p| p.id() == b.get_pipeline_id())
                .unwrap();
            pipeline_a
                .get_data()
                .data
                .index
                .cmp(&pipeline_b.get_data().data.index)
        });
    }

    fn prepare_meshes(
        &mut self,
        pipelines: &mut [ResourceRefMut<PipelineInstance>],
        materials: &mut [ResourceRefMut<MaterialInstance>],
        meshes: &mut [ResourceRefMut<MeshInstance>],
        textures: &[ResourceRefMut<TextureInstance>],
    ) {
        nrg_profiler::scoped_profile!("renderer::prepare_meshes");
        let mut material_index = 0;
        materials.iter().for_each(|material_instance| {
            if !material_instance.has_meshes() {
                return;
            }
            let pipeline = pipelines
                .iter_mut()
                .find(|p| p.id() == material_instance.get_pipeline_id())
                .unwrap();

            nrg_profiler::scoped_profile!(format!(
                "renderer::prepare_meshes_on_material[{}]",
                material_index
            )
            .as_str());

            let diffuse_texture_id = material_instance.get_diffuse_texture();
            let diffuse_texture = if diffuse_texture_id.is_nil() {
                None
            } else {
                Some(
                    self.texture_handler.get_texture(
                        textures
                            .iter()
                            .find(|&t| t.id() == diffuse_texture_id)
                            .unwrap()
                            .get_texture_index() as _,
                    ),
                )
            };
            let diffuse_texture_index: i32 = if let Some(texture) = diffuse_texture {
                texture.get_texture_index() as _
            } else {
                INVALID_INDEX
            };
            let diffuse_layer_index: i32 = if let Some(texture) = diffuse_texture {
                texture.get_layer_index() as _
            } else {
                INVALID_INDEX
            };

            material_instance.get_meshes().iter().for_each(|mesh_id| {
                let mesh_instance = meshes.iter_mut().find(|m| m.id() == *mesh_id).unwrap();
                if !mesh_instance.is_visible() {
                    return;
                }
                nrg_profiler::scoped_profile!(format!(
                    "renderer::prepare_meshes_on_material[{}]_add_mesh_to_pipeline",
                    material_index
                )
                .as_str());
                mesh_instance.process_uv_for_texture(diffuse_texture);
                pipeline.add_mesh_instance(
                    mesh_instance,
                    material_instance.get_diffuse_color(),
                    diffuse_texture_index,
                    diffuse_layer_index,
                );
            });
            material_index += 1;
        });
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.device.destroy();
        self.instance.destroy();
    }
}
