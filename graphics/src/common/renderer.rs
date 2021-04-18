use std::path::{Path, PathBuf};

use crate::fonts::font::*;
use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

use super::data_formats::*;
use super::device::*;
use super::instance::*;
use super::mesh::*;
use super::pipeline::*;
use super::render_pass::*;
use super::texture::*;
use super::viewport::*;

pub type MaterialId = Uid;
pub type PipelineId = Uid;
pub type TextureId = Uid;
pub type FontId = Uid;
pub type MeshId = Uid;
pub const INVALID_ID: Uid = Uid::nil();
pub const INVALID_INDEX: i32 = -1;

struct PipelineInstance {
    id: PipelineId,
    data: PipelineData,
    pipeline: Option<Pipeline>,
    finalized_mesh: Mesh,
    vertex_count: usize,
    indices_count: usize,
    instance_data: Vec<InstanceData>,
    instance_commands: Vec<InstanceCommand>,
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
    texture_index: i32,
}
struct MeshInstance {
    id: MeshId,
    mesh: MeshData,
    transform: Matrix4,
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
    texture_handler: TextureHandler,
    pipelines: Vec<PipelineInstance>,
    materials: Vec<MaterialInstance>,
    textures: Vec<TextureInstance>,
    fonts: Vec<FontInstance>,
}

impl Renderer {
    pub fn new(handle: &Handle, enable_debug: bool) -> Self {
        let instance = Instance::create(handle, enable_debug);
        let device = Device::create(&instance);
        let mut texture_handler = TextureHandler::create(&device);
        let texture_index = texture_handler.add_empty() as _;
        let textures = vec![TextureInstance {
            id: generate_random_uid(),
            path: PathBuf::default(),
            texture_index,
        }];
        Renderer {
            instance,
            device,
            viewport: Viewport::default(),
            scissors: Scissors::default(),
            pipelines: Vec::new(),
            materials: Vec::new(),
            fonts: Vec::new(),
            texture_handler,
            textures,
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

    pub fn add_texture(&mut self, material_id: MaterialId, texture_path: &Path) -> TextureId {
        let material_index = get_material_index_from_id(&self.materials, material_id);
        if material_index >= 0 {
            let texture_index = get_texture_index_from_path(&self.textures, texture_path);
            let texture_id = if texture_index >= 0 {
                self.textures[texture_index as usize].id
            } else {
                let id = generate_random_uid();
                self.textures.push(TextureInstance {
                    id,
                    path: PathBuf::from(texture_path),
                    texture_index: INVALID_INDEX,
                });
                id
            };
            let iter_index = self.materials[material_index as usize]
                .textures
                .iter()
                .position(|el| *el == texture_id);
            if iter_index.is_none() {
                self.materials[material_index as usize]
                    .textures
                    .push(texture_id);
            }
        }
        INVALID_ID
    }

    pub fn add_pipeline(&mut self, data: &PipelineData) {
        if !self.has_pipeline(&data.name) {
            let pipeline_id = generate_uid_from_string(data.name.as_str());
            self.pipelines.push(PipelineInstance {
                id: pipeline_id,
                data: data.clone(),
                pipeline: None,
                finalized_mesh: Mesh::create(&self.device),
                vertex_count: 0,
                indices_count: 0,
                instance_data: Vec::new(),
                instance_commands: Vec::new(),
            });
        }
    }

    pub fn add_font(&mut self, pipeline_id: PipelineId, font_path: &Path) -> FontId {
        if font_path.exists() && !self.has_font(font_path) {
            let font_id = generate_random_uid();
            let material_id = self.add_material(pipeline_id);
            self.fonts.push(FontInstance {
                id: font_id,
                path: PathBuf::from(font_path),
                material_id,
                font: Font::new(font_path),
                texture_id: INVALID_ID,
            });
            font_id
        } else {
            let index = self.get_font_index_from_path(font_path);
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
                    transform: Matrix4::IDENTITY,
                });
            return mesh_id;
        }
        INVALID_ID
    }

    pub fn update_mesh(&mut self, material_id: MaterialId, mesh_id: MeshId, transform: &Matrix4) {
        if mesh_id == INVALID_ID || material_id == INVALID_ID {
            return;
        }
        let material_index = self.get_material_index(material_id);
        if material_index >= 0 {
            let mesh_index =
                get_mesh_index_from_id(&self.materials[material_index as usize].meshes, mesh_id);
            if mesh_index >= 0 {
                self.materials[material_index as usize].meshes[mesh_index as usize].transform =
                    *transform;
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
        position: Vector2,
        scale: f32,
        color: Vector4,
        spacing: Vector2,
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
                    transform: Matrix4::IDENTITY,
                });
                return mesh_id;
            }
        }
        INVALID_ID
    }
    pub fn get_fonts_count(&self) -> usize {
        self.fonts.len()
    }
    pub fn has_font(&self, filepath: &Path) -> bool {
        self.get_font_id(filepath) != INVALID_ID
    }
    pub fn get_font_id(&self, filepath: &Path) -> FontId {
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
    pub fn get_font_index_from_path(&self, filepath: &Path) -> i32 {
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

        for (pipeline_index, pipeline_instance) in pipelines.iter_mut().enumerate() {
            if let Some(pipeline) = &mut pipeline_instance.pipeline {
                if pipeline_instance.instance_commands.is_empty() {
                    continue;
                }

                nrg_profiler::scoped_profile!(format!(
                    "renderer::draw_pipeline[{}]",
                    pipeline_index
                )
                .as_str());

                let instance_data = &pipeline_instance.instance_data;
                let instance_commands = &pipeline_instance.instance_commands;

                {
                    nrg_profiler::scoped_profile!(format!(
                        "renderer::draw_pipeline_begin[{}]",
                        pipeline_index
                    )
                    .as_str());
                    pipeline.begin(instance_commands, instance_data);
                }

                pipeline.update_uniform_buffer(Vector3::default());
                self.texture_handler.update_descriptor_sets(&pipeline);
                pipeline.bind_descriptors();

                {
                    nrg_profiler::scoped_profile!(format!(
                        "renderer::draw_pipeline_call[{}]",
                        pipeline_index
                    )
                    .as_str());
                    if pipeline_instance.vertex_count > 0 {
                        pipeline_instance
                            .finalized_mesh
                            .bind_vertices(pipeline_instance.vertex_count);
                        pipeline.bind_indirect();
                        pipeline_instance
                            .finalized_mesh
                            .bind_indices(pipeline_instance.indices_count);
                        pipeline.draw_indirect(instance_commands.len());
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
                pipeline_instance.pipeline.get_or_insert(pipeline);
            }
        });
    }

    fn load_textures(&mut self) {
        nrg_profiler::scoped_profile!("renderer::load_textures");
        let fonts = &mut self.fonts;
        let materials = &mut self.materials;
        let textures = &mut self.textures;
        let texture_handler = &mut self.texture_handler;

        fonts.iter_mut().for_each(|font_instance| {
            if font_instance.texture_id == INVALID_ID {
                let texture_index =
                    get_texture_index_from_path(textures, font_instance.path.as_path());
                if texture_index >= 0 {
                    font_instance.texture_id = textures[texture_index as usize].id;
                } else {
                    let id = add_texture_in_material(
                        font_instance.material_id,
                        materials,
                        font_instance.font.get_texture_path().as_path(),
                        textures,
                    );
                    font_instance.texture_id = id;
                }
            }
        });

        textures.iter_mut().for_each(|texture_instance| {
            if texture_instance.texture_index < 0 {
                texture_instance.texture_index =
                    texture_handler.add(texture_instance.path.as_path()) as _;
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
        let texture_handler = &mut self.texture_handler;
        let textures = &mut self.textures;
        let mut material_index = 0;
        let mut pipeline_index = INVALID_INDEX;
        let mut mesh_index = 0;
        self.materials.iter_mut().for_each(|material_instance| {
            let index = get_pipeline_index_from_id(pipelines, material_instance.pipeline_id);
            let pipeline = &mut pipelines[index as usize];
            if pipeline_index != index {
                pipeline.vertex_count = 0;
                pipeline.indices_count = 0;
                pipeline.instance_data.clear();
                pipeline.instance_commands.clear();
                pipeline_index = index;
                mesh_index = 0;
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
            let material_instance_data = &mut pipeline.instance_data;
            let material_instance_commands = &mut pipeline.instance_commands;
            let mut material_textures: Vec<i32> = Vec::new();
            material_textures.resize(material_instance.textures.len(), INVALID_INDEX);
            for (i, uid) in material_instance.textures.iter().enumerate() {
                let index = get_texture_index_from_id(textures, *uid);
                material_textures[i] = index;
            }
            material_instance
                .meshes
                .iter_mut()
                .for_each(|mesh_instance| {
                    let texture_index = if material_textures.is_empty() {
                        0
                    } else {
                        textures[material_textures[0] as usize].texture_index as _
                    };
                    for (i, v) in mesh_instance.mesh.vertices.iter().enumerate() {
                        material_mesh_data.vertices[*vertex_count + i] = *v;
                        if !material_textures.is_empty() {
                            let (u, v) = texture_handler
                                .get_texture(texture_index)
                                .convert_uv(v.tex_coord.x, v.tex_coord.y);
                            material_mesh_data.vertices[*vertex_count + i].tex_coord =
                                [u, v].into();
                        }
                    }
                    for (i, index) in mesh_instance.mesh.indices.iter().enumerate() {
                        material_mesh_data.indices[*indices_count + i] =
                            *index + *vertex_count as u32;
                    }
                    *vertex_count += mesh_instance.mesh.vertices.len();
                    *indices_count += mesh_instance.mesh.indices.len();

                    material_instance_commands.push(InstanceCommand {
                        mesh_index,
                        vertex_count: mesh_instance.mesh.vertices.len(),
                        index_count: mesh_instance.mesh.indices.len(),
                    });
                    material_instance_data.push(InstanceData {
                        transform: mesh_instance.transform,
                        diffuse_texture_index: texture_handler
                            .get_texture(texture_index)
                            .get_texture_index()
                            as _,
                        diffuse_layer_index: texture_handler
                            .get_texture(texture_index)
                            .get_layer_index() as _,
                    });

                    mesh_index += 1;
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
    INVALID_INDEX
}

fn get_material_index_from_id(materials: &[MaterialInstance], material_id: MaterialId) -> i32 {
    if let Some(index) = materials
        .iter()
        .position(|material| material.id == material_id)
    {
        return index as _;
    }
    INVALID_INDEX
}

fn get_font_index_from_id(fonts: &[FontInstance], font_id: FontId) -> i32 {
    if let Some(index) = fonts.iter().position(|font| font.id == font_id) {
        return index as _;
    }
    INVALID_INDEX
}

fn get_font_index_from_path(fonts: &[FontInstance], filepath: &Path) -> i32 {
    if let Some(index) = fonts.iter().position(|font| font.path == filepath) {
        return index as _;
    }
    INVALID_INDEX
}

fn get_texture_index_from_id(textures: &[TextureInstance], texture_id: TextureId) -> i32 {
    if let Some(index) = textures.iter().position(|texture| texture.id == texture_id) {
        return index as _;
    }
    INVALID_INDEX
}

fn get_texture_index_from_path(textures: &[TextureInstance], filepath: &Path) -> i32 {
    if let Some(index) = textures.iter().position(|texture| texture.path == filepath) {
        return index as _;
    }
    INVALID_INDEX
}

fn get_mesh_index_from_id(meshes: &[MeshInstance], mesh_id: MeshId) -> i32 {
    if let Some(index) = meshes.iter().position(|mesh| mesh.id == mesh_id) {
        return index as _;
    }
    INVALID_INDEX
}

fn add_texture_in_material(
    material_id: MaterialId,
    materials: &mut Vec<MaterialInstance>,
    texture_path: &Path,
    textures: &mut Vec<TextureInstance>,
) -> TextureId {
    let material_index = get_material_index_from_id(materials, material_id);
    if material_index >= 0 {
        let texture_index = get_texture_index_from_path(textures, texture_path);
        let texture_id = if texture_index >= 0 {
            textures[texture_index as usize].id
        } else {
            let id = generate_random_uid();
            textures.push(TextureInstance {
                id,
                path: PathBuf::from(texture_path),
                texture_index: INVALID_INDEX,
            });
            id
        };
        let iter_index = materials[material_index as usize]
            .textures
            .iter()
            .position(|el| *el == texture_id);
        if iter_index.is_none() {
            materials[material_index as usize].textures.push(texture_id);
        }
    }
    INVALID_ID
}
