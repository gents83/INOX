use std::path::Path;

use crate::{MaterialRc, MeshData, TextureInfo};
use nrg_math::{Matrix4, Vector4};
use nrg_resources::{
    DataTypeResource, Deserializable, ResourceData, ResourceId, ResourceRef, SerializableResource,
    SharedData, SharedDataRw,
};
use nrg_serialize::{generate_random_uid, INVALID_UID};

pub type MeshId = ResourceId;
pub type MeshRc = ResourceRef<MeshInstance>;

#[derive(Clone)]
pub struct MeshInstance {
    id: ResourceId,
    mesh_data: MeshData,
    material: MaterialRc,
    draw_area: Vector4, //pos (x,y) - size(z,w)
    is_visible: bool,
    is_dirty: bool,
    uv_converted: bool,
}

impl ResourceData for MeshInstance {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl Default for MeshInstance {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            mesh_data: MeshData::default(),
            material: MaterialRc::default(),
            draw_area: [0., 0., f32::MAX, f32::MAX].into(),
            is_visible: true,
            is_dirty: true,
            uv_converted: false,
        }
    }
}

impl SerializableResource for MeshInstance {
    fn path(&self) -> &Path {
        self.mesh_data.path()
    }
}

impl DataTypeResource for MeshInstance {
    type DataType = MeshData;
    fn create_from_data(shared_data: &SharedDataRw, mesh_data: Self::DataType) -> MeshRc {
        let mesh_instance = MeshInstance {
            id: generate_random_uid(),
            mesh_data,
            ..Default::default()
        };
        SharedData::add_resource(shared_data, mesh_instance)
    }
}

impl MeshInstance {
    pub fn find_from_path(shared_data: &SharedDataRw, path: &Path) -> Option<MeshRc> {
        SharedData::match_resource(shared_data, |m: &MeshInstance| m.path() == path)
    }
    pub fn get_data(&self) -> &MeshData {
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
    pub fn set_transform(&mut self, transform: Matrix4) -> &mut Self {
        self.mesh_data.transform = transform;
        self.is_dirty = true;
        self
    }
    pub fn set_mesh_data(&mut self, mesh_data: MeshData) -> &mut Self {
        self.mesh_data = mesh_data;
        self.uv_converted = false;
        self.is_dirty = true;
        self
    }
    pub fn set_material(&mut self, material: MaterialRc) -> &mut Self {
        self.material = material;
        self.is_dirty = true;
        self
    }
    pub fn material(&self) -> MaterialRc {
        self.material.clone()
    }
    pub fn mesh_data(&self) -> &MeshData {
        &self.mesh_data
    }
    pub fn transform(&self) -> Matrix4 {
        self.mesh_data.transform
    }
    pub fn draw_area(&self) -> Vector4 {
        self.draw_area
    }

    pub fn process_uv_for_texture(&mut self, texture: Option<&TextureInfo>) -> &mut Self {
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
