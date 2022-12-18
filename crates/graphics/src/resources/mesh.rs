use std::path::{Path, PathBuf};

use crate::{Material, MeshData};

use inox_bitmask::bitmask;
use inox_math::{MatBase, Matrix4, VecBase, Vector3};
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, DataTypeResourceEvent, Handle, Resource, ResourceEvent, ResourceId,
    ResourceTrait, SerializableResource, SharedData, SharedDataRc,
};
use inox_serialize::{
    inox_serializable::SerializableRegistryRc, read_from_file, Deserialize, Serialize,
    SerializeFile,
};

pub type MeshId = ResourceId;

#[bitmask]
#[repr(u32)]
pub enum MeshFlags {
    None = 0,
    Visible = 1,
    Opaque = 1 << 1,
    Tranparent = 1 << 2,
    Wireframe = 1 << 3,
    Custom = 1 << 4,
}

#[test]
fn test_serialize() {
    let flags = MeshFlags::Visible | MeshFlags::Tranparent;
    let registry = SerializableRegistryRc::default();
    let s = inox_serialize::serialize(&flags, &registry);
    println!("{}", s);
}

#[derive(Clone)]
pub struct Mesh {
    id: MeshId,
    message_hub: MessageHubRc,
    shared_data: SharedDataRc,
    path: PathBuf,
    matrix: Matrix4,
    material: Handle<Material>,
    flags: MeshFlags,
    min: Vector3,
    max: Vector3,
}

impl ResourceTrait for Mesh {
    fn is_initialized(&self) -> bool {
        self.material.is_some()
    }

    fn invalidate(&mut self) -> &mut Self {
        self.mark_as_dirty();
        self
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

    fn deserialize_data(
        path: &std::path::Path,
        registry: &SerializableRegistryRc,
        f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
        read_from_file::<Self::DataType>(path, registry, f);
    }
}

impl DataTypeResource for Mesh {
    type DataType = MeshData;

    fn new(id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            id,
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            path: PathBuf::new(),
            matrix: Matrix4::default_identity(),
            material: None,
            flags: MeshFlags::Visible | MeshFlags::Opaque,
            min: Vector3::default_zero(),
            max: Vector3::default_zero(),
        }
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
        mesh.min = data.aabb_min;
        mesh.max = data.aabb_max;
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
    pub fn min(&self) -> &Vector3 {
        &self.min
    }
    pub fn max(&self) -> &Vector3 {
        &self.max
    }
    pub fn material(&self) -> &Handle<Material> {
        &self.material
    }
    pub fn set_mesh_data(&mut self, mesh_data: MeshData) -> &mut Self {
        self.message_hub
            .send_event(DataTypeResourceEvent::<Self>::Loaded(self.id, mesh_data));
        self.mark_as_dirty();
        self
    }
    pub fn flags(&self) -> &MeshFlags {
        &self.flags
    }
    pub fn add_flag(&mut self, flag: MeshFlags) -> &mut Self {
        if !self.has_flags(flag) {
            self.flags |= flag;
            self.mark_as_dirty();
        }
        self
    }
    pub fn toggle_flag(&mut self, flag: MeshFlags) -> &mut Self {
        self.flags ^= flag;
        self.mark_as_dirty();
        self
    }
    pub fn remove_flag(&mut self, flag: MeshFlags) -> &mut Self {
        if self.has_flags(flag) {
            self.flags &= !flag;
            self.mark_as_dirty();
        }
        self
    }
    pub fn has_flags(&self, flags: MeshFlags) -> bool {
        self.flags.contains(flags)
    }
    pub fn set_flags(&mut self, flags: MeshFlags) -> &mut Self {
        if self.flags != flags {
            self.flags = flags;
            self.mark_as_dirty();
        }
        self
    }
    pub fn matrix(&self) -> Matrix4 {
        self.matrix
    }
}
