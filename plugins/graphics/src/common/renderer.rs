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
pub type FontId = usize;

struct PipelineInstance {
    id: PipelineId,
    data: PipelineData,
    pipeline: Option<Pipeline>,
}
struct MaterialInstance {
    id: MaterialId,
    pipeline_id: PipelineId,
    material: Option<Material>,
    meshes: Vec<MeshData>,
    finalized_mesh: Mesh,
}
struct FontInstance {
    id: FontId,
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
    pub fn get_font_index(&self, font_id: FontId) -> i32 {
        get_font_index_from_id(&self.fonts, font_id)
    }

    pub fn add_text(
        &mut self,
        font_id: FontId,
        text: &str,
        position: Vector2f,
        scale: f32,
        color: Vector3f,
    ) {
        let font_index = self.get_font_index(font_id);
        if font_index >= 0 {
            let materials = &self.materials;
            let font_instance = &mut self.fonts[font_index as usize];
            if let Some(font) = &mut font_instance.font {
                let material_index =
                    get_material_index_from_id(&materials, font_instance.material_id);
                if material_index >= 0 {
                    let material_instance = &mut self.materials[material_index as usize];
                    let mesh_data = font.add_text(text, position, scale, color);
                    material_instance.meshes.push(mesh_data);
                }
            }
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

        self.prepare_pipelines();
        self.prepare_materials();
        self.prepare_meshes();

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
        let material_count = self.materials.len();
        let mut last_pipeline_index = -1;
        for (i, material_instance) in self.materials.iter_mut().enumerate() {
            let pipeline_index =
                get_pipeline_index_from_id(pipelines, material_instance.pipeline_id);
            if last_pipeline_index != pipeline_index {
                if last_pipeline_index >= 0 {
                    end_pipeline(pipelines, last_pipeline_index as usize);
                }
                last_pipeline_index = pipeline_index;
                begin_pipeline(pipelines, pipeline_index as usize);
            }

            if material_instance.material.is_some() {
                let material = material_instance.material.as_ref().unwrap();
                material.update_simple();
            }
            material_instance.finalized_mesh.draw();

            if i == material_count - 1 {
                end_pipeline(pipelines, pipeline_index as usize);
            }
        }
    }

    pub fn request_font(&mut self, pipeline_id: &str, font_path: &PathBuf) -> FontId {
        let font_name = font_path.to_str().unwrap().to_string();
        if font_path.exists() && !self.has_font(&font_name) {
            let font_id = self.fonts.len();
            if let Some(pipeline_instance) = self
                .pipelines
                .iter()
                .find(|&pipeline| pipeline.data.name.eq(pipeline_id))
            {
                let material_id = self.materials.len();
                self.materials.push(MaterialInstance {
                    id: material_id,
                    material: None,
                    pipeline_id: pipeline_instance.id,
                    meshes: Vec::new(),
                    finalized_mesh: Mesh::create(&self.device),
                });
                self.fonts.push(FontInstance {
                    id: font_id,
                    name: font_name,
                    material_id,
                    font: None,
                });
            }
            font_id
        } else {
            let index = self
                .fonts
                .iter()
                .position(|font| font.name.eq(&font_name))
                .unwrap();
            self.fonts[index].id
        }
    }
}

impl Renderer {
    fn load_pipelines(&mut self) {
        let device = &mut self.device;
        let pipelines = &mut self.pipelines;
        pipelines.iter_mut().for_each(|pipeline_instance| {
            if pipeline_instance.pipeline.is_none() {
                let render_pass = RenderPass::create_default(&device);
                let pipeline = Pipeline::create(
                    &device,
                    pipeline_instance.data.vertex_shader.clone(),
                    pipeline_instance.data.fragment_shader.clone(),
                    render_pass,
                );
                pipeline_instance.pipeline.get_or_insert(pipeline);
            }
        });
    }

    fn load_fonts(&mut self) {
        let device = &mut self.device;
        let fonts = &mut self.fonts;
        let pipelines = &mut self.pipelines;
        let materials = &mut self.materials;

        fonts.iter_mut().for_each(|font_instance| {
            if font_instance.font.is_none() {
                let material_index =
                    get_material_index_from_id(&materials, font_instance.material_id);
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
                        let font = font_instance
                            .font
                            .get_or_insert(Font::new(PathBuf::from(font_instance.name.clone())));
                        mat.add_texture_from_image(font.get_bitmap());
                    }
                }
            }
        });
    }

    fn prepare_pipelines(&mut self) {
        self.pipelines.sort_by(|a, b| a.id.cmp(&b.id));
    }

    fn prepare_materials(&mut self) {
        self.materials.sort_by(|a, b| a.id.cmp(&b.id));
    }

    fn prepare_meshes(&mut self) {
        self.materials.iter_mut().for_each(|material_instance| {
            let mut unique_mesh_data = MeshData::default();
            let mut starting_index = 0;
            material_instance.meshes.iter_mut().for_each(|mesh_data| {
                mesh_data
                    .indices
                    .iter_mut()
                    .for_each(|i| *i += starting_index);
                unique_mesh_data
                    .vertices
                    .extend_from_slice(&mesh_data.vertices);
                unique_mesh_data
                    .indices
                    .extend_from_slice(&mesh_data.indices);
                starting_index += mesh_data.vertices.len() as u32;
            });
            material_instance.finalized_mesh.data = unique_mesh_data;
            material_instance.finalized_mesh.finalize();
        });
    }

    fn clear_fonts_meshes(&mut self) {
        let materials = &mut self.materials;
        self.fonts.iter_mut().for_each(|font_instance| {
            let material_index = get_material_index_from_id(&materials, font_instance.material_id);
            if material_index >= 0 {
                let material_instance = &mut materials[material_index as usize];
                material_instance.meshes.clear();
            }
            if let Some(ref mut font) = font_instance.font {
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

fn get_font_index_from_id(fonts: &[FontInstance], font_id: FontId) -> i32 {
    if let Some(index) = fonts.iter().position(|font| font.id == font_id) {
        return index as _;
    }
    -1
}

fn begin_pipeline(pipelines: &mut Vec<PipelineInstance>, index: usize) {
    let pipeline = pipelines[index].pipeline.as_mut().unwrap();
    pipeline.begin();
}

fn end_pipeline(pipelines: &mut Vec<PipelineInstance>, index: usize) {
    let pipeline = pipelines[index].pipeline.as_mut().unwrap();
    pipeline.end();
}
