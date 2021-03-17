use std::path::PathBuf;

use crate::fonts::font::*;
use image::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

use super::data_formats::*;
use super::device::*;
use super::instance::*;
use super::material::*;
use super::mesh::*;
use super::pipeline::*;
use super::render_pass::*;
use super::viewport::*;

pub type MaterialId = UID;
pub type PipelineId = UID;
pub type FontId = UID;
pub type MeshId = UID;
pub const INVALID_ID: UID = UID::nil();

struct PipelineInstance {
    id: PipelineId,
    data: PipelineData,
    pipeline: Option<Pipeline>,
}
struct MaterialInstance {
    id: MaterialId,
    pipeline_id: PipelineId,
    material: Option<Material>,
    meshes: Vec<MeshInstance>,
    finalized_mesh: Mesh,
}
struct MeshInstance {
    id: MeshId,
    mesh: MeshData,
}
struct FontInstance {
    id: FontId,
    name: String,
    material_id: MaterialId,
    font: Font,
    initialized: bool,
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
    pub fn new(handle: &Handle, enable_debug: bool) -> Self {
        let instance = Instance::create(handle, enable_debug);
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

    pub fn add_material(&mut self, pipeline_id: PipelineId) -> MaterialId {
        let material_id = generate_random_uid();
        self.materials.push(MaterialInstance {
            id: material_id,
            material: None,
            pipeline_id,
            meshes: Vec::new(),
            finalized_mesh: Mesh::create(&self.device),
        });
        material_id
    }

    pub fn remove_material(&mut self, material_id: MaterialId) {
        self.materials.retain(|material| material.id != material_id)
    }

    pub fn add_pipeline(&mut self, data: &PipelineData) {
        if !self.has_pipeline(&data.name) {
            let pipeline_id = generate_uid_from_string(data.name.as_str());
            self.pipelines.push(PipelineInstance {
                id: pipeline_id,
                data: data.clone(),
                pipeline: None,
            });
        }
    }

    pub fn add_font(&mut self, pipeline_id: PipelineId, font_path: &PathBuf) -> FontId {
        let font_name = font_path.to_str().unwrap().to_string();
        if font_path.exists() && !self.has_font(&font_name) {
            let font_id = generate_random_uid();
            let material_id = self.add_material(pipeline_id);
            self.fonts.push(FontInstance {
                id: font_id,
                name: font_name,
                material_id,
                font: Font::new(font_path.clone()),
                initialized: false,
            });
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

    pub fn add_mesh(&mut self, material_id: MaterialId, mesh_data: &MeshData) -> MeshId {
        if material_id == INVALID_ID {
            return INVALID_ID;
        }
        let material_index = self.get_material_index(material_id);
        if material_index >= 0 {
            let mesh_id = generate_random_uid();
            self.materials[material_index as usize]
                .meshes
                .push(MeshInstance {
                    id: mesh_id,
                    mesh: mesh_data.clone(),
                });
            return mesh_id;
        }
        INVALID_ID
    }

    pub fn update_mesh(&mut self, material_id: MaterialId, mesh_id: MeshId, mesh_data: &MeshData) {
        if mesh_id == INVALID_ID || material_id == INVALID_ID {
            return;
        }
        let material_index = self.get_material_index(material_id);
        if material_index >= 0 {
            let mesh_index =
                get_mesh_index_from_id(&self.materials[material_index as usize].meshes, mesh_id);
            if mesh_index >= 0 {
                self.materials[material_index as usize].meshes[mesh_index as usize].mesh =
                    mesh_data.clone();
            }
        }
    }

    pub fn remove_mesh(&mut self, material_id: MaterialId, mesh_id: MeshId) {
        let material_index = self.get_material_index(material_id);
        if material_index >= 0 {
            self.materials[material_index as usize]
                .meshes
                .retain(|mesh| mesh.id != mesh_id)
        }
    }

    pub fn add_text(
        &mut self,
        font_id: FontId,
        text: &str,
        position: Vector2f,
        scale: f32,
        color: Vector4f,
        spacing: Vector2f,
    ) -> MeshId {
        let font_index = self.get_font_index(font_id);
        if font_index >= 0 {
            let materials = &self.materials;
            let font_instance = &mut self.fonts[font_index as usize];
            let font = &mut font_instance.font;
            let material_index = get_material_index_from_id(&materials, font_instance.material_id);
            if material_index >= 0 {
                let material_instance = &mut self.materials[material_index as usize];
                let mesh_data = font.add_text(text, position, scale, color, spacing);
                let mesh_id = generate_random_uid();
                material_instance.meshes.push(MeshInstance {
                    id: mesh_id,
                    mesh: mesh_data,
                });
                return mesh_id;
            }
        }
        INVALID_ID
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
    pub fn get_font_id(&self, name: &str) -> FontId {
        if let Some(entry) = self.fonts.iter().find(|&font| font.name.eq(name)) {
            return entry.id;
        }
        INVALID_ID
    }

    pub fn get_font(&self, id: FontId) -> Option<&Font> {
        let index = get_font_index_from_id(&self.fonts, id);
        if index >= 0 {
            return Some(&self.fonts[index as usize].font);
        }
        None
    }

    pub fn get_font_material_id(&self, id: FontId) -> MaterialId {
        let index = get_font_index_from_id(&self.fonts, id);
        if index >= 0 {
            return self.fonts[index as usize].material_id;
        }
        INVALID_ID
    }

    pub fn get_default_font_id(&self) -> FontId {
        if let Some(entry) = self.fonts.first() {
            return entry.id;
        }
        INVALID_ID
    }

    pub fn get_font_index(&self, font_id: FontId) -> i32 {
        get_font_index_from_id(&self.fonts, font_id)
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

    pub fn get_pipeline_id(&self, name: &str) -> PipelineId {
        if let Some(entry) = self
            .pipelines
            .iter()
            .find(|&pipeline| pipeline.data.name.eq(name))
        {
            return entry.id;
        }
        INVALID_ID
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

    pub fn get_viewport_size(&self) -> Vector2f {
        Vector2f {
            x: self.viewport.width,
            y: self.viewport.height,
        }
    }

    pub fn begin_frame(&mut self) -> bool {
        self.load_pipelines();
        self.load_materials();
        self.load_fonts();

        self.prepare_pipelines();
        self.prepare_materials();
        self.prepare_meshes();

        let result = self.device.begin_frame();
        if !result {
            self.recreate();
        }
        result
    }

    pub fn end_frame(&mut self) -> bool {
        self.device.end_frame();

        self.clear_transient_meshes();

        let result = self.device.submit();
        if !result {
            self.recreate();
        }
        result
    }

    pub fn draw(&mut self) {
        let pipelines = &mut self.pipelines;
        let materials = &mut self.materials;

        for pipeline_instance in pipelines.iter_mut() {
            if let Some(pipeline) = &mut pipeline_instance.pipeline {
                pipeline.begin();

                for material_instance in materials.iter_mut() {
                    if material_instance.pipeline_id == pipeline_instance.id {
                        if let Some(material) = &mut material_instance.material {
                            material.update_simple();
                        }
                        material_instance.finalized_mesh.draw();
                    }
                }

                pipeline.end();
            }
        }
    }
}

impl Renderer {
    fn recreate(&mut self) {
        self.device.recreate_swap_chain();

        let pipelines = &mut self.pipelines;
        for pipeline_instance in pipelines.iter_mut() {
            if let Some(pipeline) = &mut pipeline_instance.pipeline {
                let render_pass =
                    RenderPass::create_default(&self.device, &pipeline_instance.data.data);
                pipeline.recreate(render_pass);
            }
        }
    }

    fn load_pipelines(&mut self) {
        let device = &mut self.device;
        let pipelines = &mut self.pipelines;
        pipelines.iter_mut().for_each(|pipeline_instance| {
            if pipeline_instance.pipeline.is_none() {
                let render_pass = RenderPass::create_default(&device, &pipeline_instance.data.data);
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
    fn load_materials(&mut self) {
        let device = &mut self.device;
        let pipelines = &mut self.pipelines;
        let materials = &mut self.materials;
        materials.iter_mut().for_each(|material_instance| {
            if material_instance.material.is_none() {
                let pipeline_index =
                    get_pipeline_index_from_id(&pipelines, material_instance.pipeline_id);
                if pipeline_index >= 0 {
                    let pipeline = pipelines[pipeline_index as usize]
                        .pipeline
                        .as_ref()
                        .unwrap();
                    material_instance
                        .material
                        .get_or_insert(Material::create(&device, &pipeline));
                }
            }
        });
    }

    fn load_fonts(&mut self) {
        let fonts = &mut self.fonts;
        let materials = &mut self.materials;

        fonts.iter_mut().for_each(|font_instance| {
            if !font_instance.initialized {
                let material_index =
                    get_material_index_from_id(&materials, font_instance.material_id);
                if material_index >= 0 {
                    let material_instance = &mut materials[material_index as usize];
                    if let Some(material) = &mut material_instance.material {
                        material.add_texture_from_image(font_instance.font.get_bitmap());
                        font_instance.initialized = true;
                    }
                }
            }
        });
    }

    fn prepare_pipelines(&mut self) {
        self.pipelines
            .sort_by(|a, b| a.data.data.index.cmp(&b.data.data.index));
    }

    fn prepare_materials(&mut self) {
        self.materials.sort_by(|a, b| a.id.cmp(&b.id));

        self.materials.iter_mut().for_each(|material_instance| {
            if let Some(material) = &mut material_instance.material {
                if material.get_num_textures() == 0 {
                    let image = DynamicImage::new_rgba8(1, 1);
                    material.add_texture_from_image(&image);
                }
            }
        });
    }

    fn prepare_meshes(&mut self) {
        self.materials.iter_mut().for_each(|material_instance| {
            let mut unique_mesh_data = MeshData::default();
            let mut starting_index = 0;
            material_instance.meshes.iter_mut().for_each(|mesh_data| {
                mesh_data
                    .mesh
                    .indices
                    .iter_mut()
                    .for_each(|i| *i += starting_index);
                unique_mesh_data
                    .vertices
                    .extend_from_slice(&mesh_data.mesh.vertices);
                unique_mesh_data
                    .indices
                    .extend_from_slice(&mesh_data.mesh.indices);
                starting_index += mesh_data.mesh.vertices.len() as u32;
            });
            material_instance.finalized_mesh.data = unique_mesh_data;
            material_instance.finalized_mesh.finalize();
        });
    }

    fn clear_transient_meshes(&mut self) {
        self.materials.iter_mut().for_each(|material_instance| {
            material_instance
                .meshes
                .retain(|mesh_instance| !mesh_instance.mesh.is_transient);
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

fn get_mesh_index_from_id(meshes: &[MeshInstance], mesh_id: MeshId) -> i32 {
    if let Some(index) = meshes.iter().position(|mesh| mesh.id == mesh_id) {
        return index as _;
    }
    -1
}
