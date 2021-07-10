use std::path::PathBuf;

use crate::{MeshData, Texture};
use nrg_math::{Matrix4, Vector4};
use nrg_resources::{
    DataResource, Deserializable, DynamicResource, Resource, ResourceId, ResourceTrait,
    SerializableResource, SharedDataRw,
};
use nrg_serialize::generate_random_uid;

pub type MeshId = ResourceId;
pub type MeshRc = Resource;

#[derive(Clone)]
pub struct MeshInstance {
    id: ResourceId,
    mesh_data: MeshData,
    draw_area: Vector4, //pos (x,y) - size(z,w)
    is_visible: bool,
    is_dirty: bool,
    uv_converted: bool,
}

impl ResourceTrait for MeshInstance {
    fn id(&self) -> ResourceId {
        self.id
    }
    fn path(&self) -> PathBuf {
        self.mesh_data.path().to_path_buf()
    }
}

impl Default for MeshInstance {
    fn default() -> Self {
        Self {
            id: generate_random_uid(),
            mesh_data: MeshData::default(),
            draw_area: [0., 0., f32::MAX, f32::MAX].into(),
            is_visible: true,
            is_dirty: true,
            uv_converted: false,
        }
    }
}

impl DynamicResource for MeshInstance {}

impl SerializableResource for MeshInstance {}

impl DataResource for MeshInstance {
    type DataType = MeshData;
    fn create_from_data(shared_data: &SharedDataRw, mesh_data: Self::DataType) -> MeshRc {
        let mut mesh_instance = MeshInstance::default();
        let mut data = shared_data.write().unwrap();
        mesh_instance.mesh_data = mesh_data;
        data.add_resource(mesh_instance)
    }
}

impl MeshInstance {
    pub fn get_data(&self) -> &MeshData {
        &self.mesh_data
    }
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
    pub fn set_visible(&mut self, is_visible: bool) {
        self.is_visible = is_visible;
        self.is_dirty = true;
    }
    pub fn set_draw_area(&mut self, draw_area: Vector4) {
        self.draw_area = draw_area;
        self.is_dirty = true;
    }
    pub fn set_transform(&mut self, transform: Matrix4) {
        self.mesh_data.transform = transform;
        self.is_dirty = true;
    }
    pub fn set_mesh_data(&mut self, mesh_data: MeshData) {
        self.mesh_data = mesh_data;
        self.uv_converted = false;
        self.is_dirty = true;
    }
    pub fn transform(&self) -> &Matrix4 {
        &self.mesh_data.transform
    }
    pub fn draw_area(&self) -> Vector4 {
        self.draw_area
    }

    pub fn process_uv_for_texture(&mut self, texture: Option<&Texture>) -> &mut Self {
        if !self.uv_converted {
            self.uv_converted = true;
            for v in self.mesh_data.vertices.iter_mut() {
                let tex_coord = &mut v.tex_coord;
                if let Some(texture) = texture {
                    let (u, v) = texture.convert_uv(tex_coord.x, tex_coord.y);
                    *tex_coord = [u, v].into();
                } else {
                    *tex_coord = [0., 0.].into();
                }
            }
        }
        self
    }
}
