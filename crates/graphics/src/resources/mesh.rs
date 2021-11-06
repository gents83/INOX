use std::path::{Path, PathBuf};

use crate::{Material, MeshCategoryId, MeshData, INVALID_INDEX};
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
    data: MeshData,
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
            data: MeshData::default(),
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

    fn extension() -> &'static str {
        "mesh_data"
    }
}

impl DataTypeResource for Mesh {
    type DataType = MeshData;
    fn is_initialized(&self) -> bool {
        !self.data.vertices.is_empty()
    }

    fn invalidate(&mut self) {
        self.data = MeshData::default();
    }
    fn deserialize_data(path: &Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        _id: ResourceId,
        data: Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let material = if !data.material.to_str().unwrap_or_default().is_empty() {
            let material = Material::request_load(
                shared_data,
                global_messenger,
                data.material.as_path(),
                None,
            );
            Some(material)
        } else {
            None
        };
        Self {
            data,
            material,
            ..Default::default()
        }
    }
}

impl Mesh {
    pub fn find_from_path(shared_data: &SharedDataRc, path: &Path) -> Handle<Self> {
        SharedData::match_resource(shared_data, |m: &Mesh| m.path() == path)
    }
    pub fn mesh_data(&self) -> &MeshData {
        &self.data
    }
    pub fn is_visible(&self) -> bool {
        self.is_visible && !self.data.vertices.is_empty() && !self.data.indices.is_empty()
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
        self.data = mesh_data;
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
        &self.data.mesh_category_identifier
    }
}
