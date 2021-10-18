use std::path::{Path, PathBuf};

use crate::{Material, MeshCategoryId, MeshData, TextureInfo, INVALID_INDEX};
use nrg_math::{MatBase, Matrix4, Vector4};
use nrg_messenger::MessengerRw;
use nrg_resources::{
    DataTypeResource, Handle, Resource, ResourceId, SerializableResource, SharedData, SharedDataRc,
};
use nrg_serialize::read_from_file;

pub type MeshId = ResourceId;

#[derive(Clone)]
pub struct Mesh {
    path: PathBuf,
    mesh_data: MeshData,
    matrix: Matrix4,
    material: Handle<Material>,
    draw_area: Vector4, //pos (x,y) - size(z,w)
    is_visible: bool,
    is_dirty: bool,
    uv_converted: bool,
    draw_index: i32,
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            mesh_data: MeshData::default(),
            matrix: Matrix4::default_identity(),
            material: None,
            draw_area: [0., 0., f32::MAX, f32::MAX].into(),
            is_visible: true,
            is_dirty: true,
            uv_converted: false,
            draw_index: INVALID_INDEX,
        }
    }
}

impl SerializableResource for Mesh {
    fn set_path(&mut self, path: &Path) {
        self.path = path.to_path_buf();
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn is_matching_extension(path: &Path) -> bool {
        const MESH_EXTENSION: &str = "mesh_data";

        if let Some(ext) = path.extension().unwrap().to_str() {
            return ext == MESH_EXTENSION;
        }
        false
    }
}

impl DataTypeResource for Mesh {
    type DataType = MeshData;
    fn is_initialized(&self) -> bool {
        !self.mesh_data.vertices.is_empty()
    }

    fn invalidate(&mut self) {
        self.mesh_data = MeshData::default();
    }
    fn deserialize_data(path: &Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }
    fn create_from_data(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        id: MeshId,
        mesh_data: Self::DataType,
    ) -> Resource<Self> {
        let material = if !mesh_data.material.to_str().unwrap_or_default().is_empty() {
            let material = Material::load_from_file(
                shared_data,
                global_messenger,
                mesh_data.material.as_path(),
            );
            Some(material)
        } else {
            None
        };
        let mesh = Self {
            mesh_data,
            material,
            ..Default::default()
        };
        SharedData::add_resource(shared_data, id, mesh)
    }
}

impl Mesh {
    pub fn find_from_path(shared_data: &SharedDataRc, path: &Path) -> Handle<Self> {
        SharedData::match_resource(shared_data, |m: &Mesh| m.path() == path)
    }
    pub fn mesh_data(&self) -> &MeshData {
        &self.mesh_data
    }
    pub fn is_visible(&self) -> bool {
        self.is_visible && !self.mesh_data.vertices.is_empty() && !self.mesh_data.indices.is_empty()
    }
    pub fn set_visible(&mut self, is_visible: bool) -> &mut Self {
        self.is_visible = is_visible;
        self.is_dirty = true;
        self
    }
    pub fn set_draw_area(&mut self, draw_area: Vector4) -> &mut Self {
        self.draw_area = draw_area;
        self.is_dirty = true;
        self
    }
    pub fn set_matrix(&mut self, transform: Matrix4) -> &mut Self {
        self.matrix = transform;
        self.is_dirty = true;
        self
    }
    pub fn set_mesh_data(&mut self, mesh_data: MeshData) -> &mut Self {
        self.mesh_data = mesh_data;
        self.uv_converted = false;
        self.is_dirty = true;
        self
    }
    pub fn set_material(&mut self, material: Resource<Material>) -> &mut Self {
        self.material = Some(material);
        self.is_dirty = true;
        self
    }
    pub fn material(&self) -> &Handle<Material> {
        &self.material
    }
    pub fn draw_index(&self) -> i32 {
        self.draw_index
    }
    pub fn set_draw_index(&mut self, draw_index: u32) {
        self.draw_index = draw_index as _;
    }
    pub fn matrix(&self) -> Matrix4 {
        self.matrix
    }
    pub fn draw_area(&self) -> Vector4 {
        self.draw_area
    }

    pub fn category_identifier(&self) -> &MeshCategoryId {
        &self.mesh_data.mesh_category_identifier
    }

    pub fn process_uv_for_texture(&mut self, texture: Option<&TextureInfo>) -> &mut Self {
        if let Some(texture) = texture {
            if !self.uv_converted {
                nrg_profiler::scoped_profile!("Texture::process_uv_for_texture");
                self.uv_converted = true;
                for v in self.mesh_data.vertices.iter_mut() {
                    let tex_coord = &mut v.tex_coord;
                    let (u, v) = texture.convert_uv(tex_coord.x, tex_coord.y);
                    *tex_coord = [u, v].into();
                }
            }
        }
        self
    }
}
