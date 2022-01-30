use std::path::{Path, PathBuf};

use crate::{Material, MeshData, INVALID_INDEX};
use sabi_math::{MatBase, Matrix4, Vector4};
use sabi_messenger::MessengerRw;
use sabi_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedData, SharedDataRc,
};
use sabi_serialize::{read_from_file, SerializeFile};

pub type MeshId = ResourceId;

#[derive(Clone)]
pub struct OnMeshCreateData {
    pub parent_matrix: Matrix4,
}

#[derive(Clone)]
pub struct Mesh {
    path: PathBuf,
    data: MeshData,
    matrix: Matrix4,
    material: Handle<Material>,
    draw_area: Vector4, //pos (x,y) - size(z,w)
    is_visible: bool,
    is_dirty: bool,
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
            is_dirty: false,
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
        MeshData::extension()
    }
}

impl DataTypeResource for Mesh {
    type DataType = MeshData;
    type OnCreateData = OnMeshCreateData;

    fn is_initialized(&self) -> bool {
        !self.data.vertices.is_empty()
    }

    fn invalidate(&mut self) -> &mut Self {
        self.data = MeshData::default();
        self.is_dirty = true;
        self
    }
    fn deserialize_data(path: &Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }
    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _id: &MeshId,
        on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
        if let Some(on_create_data) = on_create_data {
            self.set_matrix(on_create_data.parent_matrix);
        }
    }
    fn on_destroy(&mut self, _shared_data: &SharedData, _id: &MeshId) {}

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
            is_dirty: true,
            ..Default::default()
        }
    }
}

impl Mesh {
    pub fn init(&mut self) {
        self.is_dirty = false;
    }
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }
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
    pub fn set_draw_index(&mut self, draw_index: u32) -> &mut Self {
        self.draw_index = draw_index as _;
        self
    }
    pub fn matrix(&self) -> Matrix4 {
        self.matrix
    }
    pub fn draw_area(&self) -> Vector4 {
        self.draw_area
    }
}
