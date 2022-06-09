use std::path::{Path, PathBuf};

use crate::{Material, MeshData};
use inox_math::{MatBase, Matrix4, Vector4};
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, DataTypeResourceEvent, Handle, Resource, ResourceEvent, ResourceId,
    ResourceTrait, SerializableResource, SharedData, SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file, SerializeFile};

pub type MeshId = ResourceId;

pub const MESH_FLAGS_NONE: u32 = 0;
pub const MESH_FLAGS_VISIBLE: u32 = 1;
pub const MESH_FLAGS_OPAQUE: u32 = 1 << 1;
pub const MESH_FLAGS_TRANSPARENT: u32 = 1 << 2;
pub const MESH_FLAGS_WIREFRAME: u32 = 1 << 3;
pub const MESH_FLAGS_DEBUG: u32 = 1 << 4;
pub const MESH_FLAGS_UI: u32 = 1 << 5;

#[derive(Clone)]
pub struct OnMeshCreateData {
    pub parent_matrix: Matrix4,
}

#[derive(Clone)]
pub struct Mesh {
    id: MeshId,
    message_hub: MessageHubRc,
    shared_data: SharedDataRc,
    path: PathBuf,
    matrix: Matrix4,
    material: Handle<Material>,
    draw_area: Vector4, //pos (x,y) - size(z,w)
    flags: u32,
}

impl ResourceTrait for Mesh {
    type OnCreateData = OnMeshCreateData;

    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &MeshId,
        on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
        if let Some(on_create_data) = on_create_data {
            self.set_matrix(on_create_data.parent_matrix);
        }
    }
    fn on_destroy(&mut self, _shared_data: &SharedData, _message_hub: &MessageHubRc, _id: &MeshId) {
    }
    fn on_copy(&mut self, other: &Self)
    where
        Self: Sized,
    {
        *self = other.clone();
    }
}

impl SerializableResource for Mesh {
    fn set_path(&mut self, path: &Path) -> &mut Self {
        self.path = path.to_path_buf();
        self
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
    type OnCreateData = <Self as ResourceTrait>::OnCreateData;

    fn new(id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            id,
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            path: PathBuf::new(),
            matrix: Matrix4::default_identity(),
            material: None,
            draw_area: [0., 0., f32::MAX, f32::MAX].into(),
            flags: MESH_FLAGS_VISIBLE | MESH_FLAGS_OPAQUE,
        }
    }
    fn is_initialized(&self) -> bool {
        self.material.is_some()
    }

    fn invalidate(&mut self) -> &mut Self {
        self.mark_as_dirty();
        self
    }
    fn deserialize_data(
        path: &std::path::Path,
        registry: &SerializableRegistryRc,
        f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
        read_from_file::<Self::DataType>(path, registry, f);
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: ResourceId,
        data: &Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let material = if !data.material.to_str().unwrap_or_default().is_empty() {
            let material =
                Material::request_load(shared_data, message_hub, data.material.as_path(), None);
            Some(material)
        } else {
            None
        };
        let mut mesh = Mesh::new(id, shared_data, message_hub);
        mesh.material = material;
        mesh
    }
}

impl Mesh {
    pub fn mark_as_dirty(&self) -> &Self {
        self.message_hub
            .send_event(ResourceEvent::<Self>::Changed(self.id));
        self
    }
    pub fn find_from_path(shared_data: &SharedDataRc, path: &Path) -> Handle<Self> {
        SharedData::match_resource(shared_data, |m: &Mesh| m.path() == path)
    }
    pub fn set_draw_area(&mut self, draw_area: Vector4) -> &mut Self {
        if self.draw_area != draw_area {
            self.draw_area = draw_area;
            self.mark_as_dirty();
        }
        self
    }
    pub fn set_matrix(&mut self, transform: Matrix4) -> &mut Self {
        if self.matrix != transform {
            self.matrix = transform;
            self.mark_as_dirty();
        }
        self
    }
    pub fn set_material(&mut self, material: Resource<Material>) -> &mut Self {
        if self.material.is_none() || self.material.as_ref().unwrap().id() != material.id() {
            self.material = Some(material);
            self.mark_as_dirty();
        }
        self
    }
    pub fn material(&self) -> &Handle<Material> {
        &self.material
    }
    pub fn set_mesh_data(&mut self, mesh_data: MeshData) -> &mut Self {
        self.message_hub
            .send_event(DataTypeResourceEvent::<Self>::Loaded(self.id, mesh_data));
        self
    }
    pub fn flags(&self) -> u32 {
        self.flags
    }
    pub fn add_flag(&mut self, flag: u32) -> &mut Self {
        if !self.has_flags(flag) {
            self.flags |= flag;
            self.mark_as_dirty();
        }
        self
    }
    pub fn toggle_flag(&mut self, flag: u32) -> &mut Self {
        self.flags ^= flag;
        self.mark_as_dirty();
        self
    }
    pub fn remove_flag(&mut self, flag: u32) -> &mut Self {
        if self.has_flags(flag) {
            self.flags &= !flag;
            self.mark_as_dirty();
        }
        self
    }
    pub fn has_flags(&self, flags: u32) -> bool {
        self.flags & flags == flags
    }
    pub fn set_flags(&mut self, flags: u32) -> &mut Self {
        if self.flags != flags {
            self.flags = flags;
            self.mark_as_dirty();
        }
        self
    }
    pub fn matrix(&self) -> Matrix4 {
        self.matrix
    }
    pub fn draw_area(&self) -> Vector4 {
        self.draw_area
    }
}
