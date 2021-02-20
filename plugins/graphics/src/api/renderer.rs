use std::path::PathBuf;

use crate::config::*;
use crate::fonts::font::*;
use nrg_math::*;
use nrg_platform::Handle;

use super::data_formats::*;
use super::device::*;
use super::instance::*;
use super::material::*;
use super::mesh::*;
use super::pipeline::*;
use super::render_pass::*;
use super::viewport::*;

pub type MaterialId = usize;
pub type PipelineId = usize;
struct PipelineInstance {
    id: PipelineId,
    data: PipelineData,
    pipeline: Option<Pipeline>,
}
struct MaterialInstance {
    id: MaterialId,
    pipeline_id: PipelineId,
    material: Option<Material>,
    meshes: Vec<Mesh>,
}
struct FontInstance {
    name: String,
    material_id: MaterialId,
    font: Option<Font>,
}

pub struct Renderer {
    pub instance: Instance,
    pub device: Device,
    viewport: Viewport,
    scissors: Scissors,
    pipelines: Vec<PipelineInstance>,
    materials: Vec<MaterialInstance>,
    fonts: Vec<FontInstance>,
}

impl Renderer {
    pub fn new(handle: &Handle, config: &Config) -> Self {
        let instance = Instance::create(handle, config.vk_data.debug_validation_layers);
        let device = Device::create(&instance);
        Renderer {
            instance,
            device,
            viewport: Viewport::default(),
            scissors: Scissors::default(),
            pipelines: Vec::new(),
            materials: Vec::new(),
            fonts: Vec::new(),
        }
    }

    pub fn get_fonts_count(&self) -> usize {
        self.fonts.len()
    }
    pub fn has_font(&self, name: &str) -> bool {
        if let Some(_entry) = self.fonts.iter().find(|&font| font.name.eq(name)) {
            return true;
        }
        false
    }

    pub fn add_text(
        &mut self,
        font_index: usize,
        text: &str,
        position: Vector2f,
        scale: f32,
        color: Vector3f,
    ) {
        if font_index >= self.get_fonts_count() {
            return;
        }
        if let Some(font) = &mut self.fonts[font_index].font {
            font.add_text(text, position, scale, color);
        }
    }

    pub fn add_pipeline(&mut self, data: &PipelineData) {
        if !self.has_pipeline(&data.name) {
            let pipeline_id = self.pipelines.len();
            self.pipelines.push(PipelineInstance {
                id: pipeline_id,
                data: data.clone(),
                pipeline: None,
            });
        }
    }

    pub fn has_pipeline(&self, name: &str) -> bool {
        if let Some(_entry) = self
            .pipelines
            .iter()
            .find(|&pipeline| pipeline.data.name.eq(name))
        {
            return true;
        }
        false
    }

    pub fn get_pipeline_index(&self, pipeline_id: PipelineId) -> i32 {
        get_pipeline_index_from_id(&self.pipelines, pipeline_id)
    }

    pub fn get_material_index(&self, material_id: MaterialId) -> i32 {
        get_material_index_from_id(&self.materials, material_id)
    }

    pub fn set_viewport_size(&mut self, size: Vector2u) -> &mut Self {
        self.viewport.width = size.x as _;
        self.viewport.height = size.y as _;
        self.scissors.width = self.viewport.width;
        self.scissors.height = self.viewport.height;
        self
    }

    pub fn begin_frame(&mut self) -> bool {
        self.load_pipelines();
        self.load_fonts();
        self.create_fonts_meshes();

        self.device.begin_frame()
    }

    pub fn end_frame(&mut self) -> bool {
        self.device.end_frame();

        self.clear_fonts_meshes();

        //TEMP
        self.device.submit()
    }

    pub fn draw(&mut self) {
        let pipelines = &mut self.pipelines;

        for material in self.materials.iter_mut() {
            let pipeline_index = get_pipeline_index_from_id(pipelines, material.pipeline_id);
            if pipeline_index >= 0 {
                let pipeline = pipelines[pipeline_index as usize]
                    .pipeline
                    .as_mut()
                    .unwrap();

                pipeline.begin();

                if material.material.is_some() {
                    let mat = material.material.as_ref().unwrap();
                    mat.update_simple();
                }

                for mesh in material.meshes.iter() {
                    mesh.draw();
                }

                pipeline.end();
            }
        }
    }

    pub fn request_font(&mut self, pipeline_id: &str, font_path: &PathBuf) -> usize {
        let font_name = font_path.to_str().unwrap().to_string();
        if font_path.exists() && !self.has_font(&font_name) {
            if let Some(loaded_pipeline) = self
                .pipelines
                .iter()
                .find(|&pipeline| pipeline.data.name.eq(pipeline_id))
            {
                let material_id = self.materials.len();
                self.materials.push(MaterialInstance {
                    id: material_id,
                    material: None,
                    pipeline_id: loaded_pipeline.id,
                    meshes: Vec::new(),
                });
                self.fonts.push(FontInstance {
                    name: font_name,
                    material_id,
                    font: None,
                });
            }
            self.fonts.len()
        } else {
            self.fonts
                .iter()
                .position(|font| font.name.eq(&font_name))
                .unwrap()
        }
    }
}

impl Renderer {
    fn load_pipelines(&mut self) {
        let device = &mut self.device;
        let pipelines = &mut self.pipelines;
        pipelines.iter_mut().for_each(|loaded_pipeline| {
            if loaded_pipeline.pipeline.is_none() {
                let render_pass = RenderPass::create_default(&device);
                let pipeline = Pipeline::create(
                    &device,
                    loaded_pipeline.data.vertex_shader.clone(),
                    loaded_pipeline.data.fragment_shader.clone(),
                    render_pass,
                );
                loaded_pipeline.pipeline.get_or_insert(pipeline);
            }
        });
    }

    fn load_fonts(&mut self) {
        let device = &mut self.device;
        let fonts = &mut self.fonts;
        let pipelines = &mut self.pipelines;
        let materials = &mut self.materials;

        fonts.iter_mut().for_each(|loaded_font| {
            if loaded_font.font.is_none() {
                let material_index =
                    get_material_index_from_id(&materials, loaded_font.material_id);
                if material_index >= 0 {
                    let material = &mut materials[material_index as usize];
                    let pipeline_index =
                        get_pipeline_index_from_id(&pipelines, material.pipeline_id);
                    if pipeline_index >= 0 {
                        let pipeline = pipelines[pipeline_index as usize]
                            .pipeline
                            .as_ref()
                            .unwrap();
                        let mat = material
                            .material
                            .get_or_insert(Material::create(&device, &pipeline));
                        let font = loaded_font.font.get_or_insert(Font::new(
                            &device,
                            PathBuf::from(loaded_font.name.clone()),
                        ));
                        mat.add_texture_from_image(font.get_bitmap());
                    }
                }
            }
        });
    }

    fn create_fonts_meshes(&mut self) {
        let materials = &mut self.materials;
        self.fonts.iter_mut().for_each(|loaded_font| {
            let material_index = get_material_index_from_id(&materials, loaded_font.material_id);
            if material_index >= 0 {
                let material = &mut materials[material_index as usize];
                loaded_font
                    .font
                    .as_mut()
                    .unwrap()
                    .create_meshes(&mut material.meshes);
            }
        });
    }

    fn clear_fonts_meshes(&mut self) {
        self.fonts.iter_mut().for_each(|loaded_font| {
            if let Some(ref mut font) = loaded_font.font {
                font.clear();
            }
        });
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.device.destroy();
        self.instance.destroy();
    }
}

fn get_pipeline_index_from_id(pipelines: &[PipelineInstance], pipeline_id: PipelineId) -> i32 {
    if let Some(index) = pipelines
        .iter()
        .position(|pipeline| pipeline.id == pipeline_id)
    {
        return index as _;
    }
    -1
}

fn get_material_index_from_id(materials: &[MaterialInstance], material_id: MaterialId) -> i32 {
    if let Some(index) = materials
        .iter()
        .position(|material| material.id == material_id)
    {
        return index as _;
    }
    -1
}
