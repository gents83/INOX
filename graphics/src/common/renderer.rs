use std::path::PathBuf;

use crate::fonts::font::*;
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
use super::texture::*;
use super::viewport::*;

pub type MaterialId = UID;
pub type PipelineId = UID;
pub type TextureId = UID;
pub type FontId = UID;
pub type MeshId = UID;
pub const INVALID_ID: UID = UID::nil();

struct PipelineInstance {
    id: PipelineId,
    data: PipelineData,
    pipeline: Option<Pipeline>,
    material: Option<Material>,
    finalized_mesh: Mesh,
    vertex_count: usize,
    indices_count: usize,
}
struct MaterialInstance {
    id: MaterialId,
    pipeline_id: PipelineId,
    meshes: Vec<MeshInstance>,
    textures: Vec<TextureId>,
}
struct TextureInstance {
    id: TextureId,
    path: PathBuf,
    texture: Texture,
}
struct MeshInstance {
    id: MeshId,
    mesh: MeshData,
}
struct FontInstance {
    id: FontId,
    path: PathBuf,
    material_id: MaterialId,
    texture_id: TextureId,
    font: Font,
}

pub struct Renderer {
    pub instance: Instance,
    pub device: Device,
    viewport: Viewport,
    scissors: Scissors,
    pipelines: Vec<PipelineInstance>,
    materials: Vec<MaterialInstance>,
    textures: Vec<TextureInstance>,
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
            textures: Vec::new(),
            fonts: Vec::new(),
        }
    }

    pub fn add_material(&mut self, pipeline_id: PipelineId) -> MaterialId {
        let material_id = generate_random_uid();
        self.materials.push(MaterialInstance {
            id: material_id,
            pipeline_id,
            meshes: Vec::new(),
            textures: Vec::new(),
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
                material: None,
                finalized_mesh: Mesh::create(&self.device),
                vertex_count: 0,
                indices_count: 0,
            });
        }
    }

    pub fn add_font(&mut self, pipeline_id: PipelineId, font_path: &PathBuf) -> FontId {
        if font_path.exists() && !self.has_font(font_path.clone()) {
            let font_id = generate_random_uid();
            let material_id = self.add_material(pipeline_id);
            self.fonts.push(FontInstance {
                id: font_id,
                path: font_path.clone(),
                material_id,
                font: Font::new(font_path.clone()),
                texture_id: INVALID_ID,
            });
            font_id
        } else {
            let index = self.get_font_index_from_path(font_path.clone());
            self.fonts[index as usize].id
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

    pub fn get_textures_count(&self) -> usize {
        self.textures.len()
    }
    pub fn has_texture(&self, filepath: PathBuf) -> bool {
        self.get_texture_id(filepath) != INVALID_ID
    }
    pub fn get_texture_id(&self, filepath: PathBuf) -> TextureId {
        let index = get_texture_index_from_path(&self.textures, filepath);
        if index >= 0 {
            return self.textures[index as usize].id;
        }
        INVALID_ID
    }

    pub fn get_texture(&self, id: TextureId) -> Option<&Texture> {
        let index = get_texture_index_from_id(&self.textures, id);
        if index >= 0 {
            return Some(&self.textures[index as usize].texture);
        }
        None
    }

    pub fn get_fonts_count(&self) -> usize {
        self.fonts.len()
    }
    pub fn has_font(&self, filepath: PathBuf) -> bool {
        self.get_font_id(filepath) != INVALID_ID
    }
    pub fn get_font_id(&self, filepath: PathBuf) -> FontId {
        let index = get_font_index_from_path(&self.fonts, filepath);
        if index >= 0 {
            return self.fonts[index as usize].id;
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
    pub fn get_font_index_from_path(&self, filepath: PathBuf) -> i32 {
        get_font_index_from_path(&self.fonts, filepath)
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
        nrg_profiler::scoped_profile!("renderer::begin_frame");
        self.load_pipelines();
        self.load_textures();

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
        nrg_profiler::scoped_profile!("renderer::end_frame");
        self.device.end_frame();

        self.clear_transient_meshes();

        let result = self.device.submit();
        if !result {
            self.recreate();
        }
        result
    }

    pub fn draw(&mut self) {
        nrg_profiler::scoped_profile!("renderer::draw");
        let pipelines = &mut self.pipelines;
        let materials = &mut self.materials;
        let textures = &mut self.textures;

        for (pipeline_index, pipeline_instance) in pipelines.iter_mut().enumerate() {
            if let Some(pipeline) = &mut pipeline_instance.pipeline {
                nrg_profiler::scoped_profile!(format!(
                    "renderer::draw_pipeline[{}]",
                    pipeline_index
                )
                .as_str());

                {
                    nrg_profiler::scoped_profile!(format!(
                        "renderer::draw_pipeline_begin[{}]",
                        pipeline_index
                    )
                    .as_str());
                    pipeline.begin();
                }

                if let Some(material) = &mut pipeline_instance.material {
                    for (material_index, material_instance) in materials.iter_mut().enumerate() {
                        if !material_instance.textures.is_empty() {
                            nrg_profiler::scoped_profile!(format!(
                                "renderer::update_material[{}]",
                                material_index
                            )
                            .as_str());
                            let mut material_textures = Vec::new();
                            for texture_id in material_instance.textures.iter() {
                                let texture_index =
                                    get_texture_index_from_id(textures, *texture_id);
                                if texture_index >= 0 {
                                    material_textures
                                        .push(&textures[texture_index as usize].texture);
                                }
                            }
                            material.update_simple(&material_textures);
                        }
                    }
                }

                {
                    nrg_profiler::scoped_profile!(format!(
                        "renderer::draw_pipeline_call[{}]",
                        pipeline_index
                    )
                    .as_str());
                    pipeline_instance.finalized_mesh.draw(
                        pipeline_instance.vertex_count,
                        pipeline_instance.indices_count,
                    );
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

    pub fn recreate(&mut self) {
        nrg_profiler::scoped_profile!("renderer::recreate");
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
}

impl Renderer {
    fn load_pipelines(&mut self) {
        nrg_profiler::scoped_profile!("renderer::load_pipelines");
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
                pipeline_instance
                    .material
                    .get_or_insert(Material::create(&device, &pipeline));
                pipeline_instance.pipeline.get_or_insert(pipeline);
            }
        });
    }

    fn load_textures(&mut self) {
        nrg_profiler::scoped_profile!("renderer::load_textures");
        let fonts = &mut self.fonts;
        let textures = &mut self.textures;
        let materials = &mut self.materials;
        let device = &self.device;

        if textures.is_empty() {
            textures.push(TextureInstance {
                id: generate_random_uid(),
                texture: Texture::empty(&device),
                path: PathBuf::default(),
            });
        }

        fonts.iter_mut().for_each(|font_instance| {
            if font_instance.texture_id == INVALID_ID {
                let texture_index =
                    get_texture_index_from_path(textures, font_instance.path.clone());
                if texture_index >= 0 {
                    font_instance.texture_id = textures[texture_index as usize].id;
                } else {
                    let id = generate_random_uid();
                    let path = font_instance.path.clone();
                    textures.push(TextureInstance {
                        id,
                        texture: Texture::create(&device, font_instance.font.get_texture_path()),
                        path,
                    });
                    font_instance.texture_id = id;
                }
                let material_index =
                    get_material_index_from_id(&materials, font_instance.material_id);
                if material_index >= 0 {
                    let material_instance = &mut materials[material_index as usize];
                    if !material_instance
                        .textures
                        .contains(&font_instance.texture_id)
                    {
                        material_instance.textures.push(font_instance.texture_id)
                    }
                }
            }
        });
    }

    fn prepare_pipelines(&mut self) {
        nrg_profiler::scoped_profile!("renderer::prepare_pipelines");
        self.pipelines
            .sort_by(|a, b| a.data.data.index.cmp(&b.data.data.index));
    }

    fn prepare_materials(&mut self) {
        nrg_profiler::scoped_profile!("renderer::prepare_materials");
        let pipelines = &self.pipelines;
        self.materials.sort_by(|a, b| {
            let pipeline_a =
                &pipelines[get_pipeline_index_from_id(pipelines, a.pipeline_id) as usize];
            let pipeline_b =
                &pipelines[get_pipeline_index_from_id(pipelines, b.pipeline_id) as usize];
            pipeline_a.data.data.index.cmp(&pipeline_b.data.data.index)
        });
    }

    fn prepare_meshes(&mut self) {
        nrg_profiler::scoped_profile!("renderer::prepare_meshes");
        let pipelines = &mut self.pipelines;
        let mut material_index = 0;
        let mut pipeline_index = -1;
        self.materials.iter_mut().for_each(|material_instance| {
            let index = get_pipeline_index_from_id(pipelines, material_instance.pipeline_id);
            let pipeline = &mut pipelines[index as usize];
            if pipeline_index != index {
                pipeline.vertex_count = 0;
                pipeline.indices_count = 0;
                pipeline_index = index;
            }
            if pipeline.finalized_mesh.data.vertices.is_empty() {
                nrg_profiler::scoped_profile!(format!(
                    "renderer::fill_mesh_with_max_buffers[{}]",
                    material_index
                )
                .as_str());
                pipeline.finalized_mesh.fill_mesh_with_max_buffers();
            }
            nrg_profiler::scoped_profile!(format!(
                "renderer::prepare_meshes_on_material[{}]",
                material_index
            )
            .as_str());
            let vertex_count = &mut pipeline.vertex_count;
            let indices_count = &mut pipeline.indices_count;
            let material_mesh_data = &mut pipeline.finalized_mesh.data;
            material_instance.meshes.iter_mut().for_each(|mesh_data| {
                let (_left, right) = material_mesh_data.vertices.split_at_mut(*vertex_count);
                let (left, _right) = right.split_at_mut(mesh_data.mesh.vertices.len());
                left.clone_from_slice(&mesh_data.mesh.vertices);
                for (i, index) in mesh_data.mesh.indices.iter().enumerate() {
                    material_mesh_data.indices[*indices_count + i] = *index + *vertex_count as u32;
                }
                *vertex_count += mesh_data.mesh.vertices.len();
                *indices_count += mesh_data.mesh.indices.len();
            });
            pipeline.finalized_mesh.data.compute_center();
            material_index += 1;
        });
    }

    fn clear_transient_meshes(&mut self) {
        nrg_profiler::scoped_profile!("renderer::clear_transient_meshes");
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

fn get_font_index_from_path(fonts: &[FontInstance], filepath: PathBuf) -> i32 {
    if let Some(index) = fonts.iter().position(|font| font.path == filepath) {
        return index as _;
    }
    -1
}

fn get_texture_index_from_id(textures: &[TextureInstance], texture_id: TextureId) -> i32 {
    if let Some(index) = textures.iter().position(|texture| texture.id == texture_id) {
        return index as _;
    }
    -1
}

fn get_texture_index_from_path(textures: &[TextureInstance], filepath: PathBuf) -> i32 {
    if let Some(index) = textures.iter().position(|texture| texture.path == filepath) {
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
