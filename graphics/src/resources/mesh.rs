use std::path::PathBuf;

use crate::{MeshData, Texture};
use nrg_math::{MatBase, Matrix4, Vector4};
use nrg_resources::{ResourceId, ResourceTrait, SharedData, SharedDataRw};
use nrg_serialize::generate_random_uid;

pub type MeshId = ResourceId;

#[derive(Clone)]
pub struct MeshInstance {
    id: ResourceId,
    mesh_data: MeshData,
    draw_area: Vector4, //pos (x,y) - size(z,w)
    transform: Matrix4,
    is_visible: bool,
    is_dirty: bool,
    uv_converted: bool,
}

impl ResourceTrait for MeshInstance {
    fn id(&self) -> ResourceId {
        self.id
    }
    fn path(&self) -> PathBuf {
        PathBuf::default()
    }
}

impl Default for MeshInstance {
    fn default() -> Self {
        Self {
            id: generate_random_uid(),
            mesh_data: MeshData::default(),
            draw_area: [0., 0., f32::MAX, f32::MAX].into(),
            transform: Matrix4::default_identity(),
            is_visible: true,
            is_dirty: true,
            uv_converted: false,
        }
    }
}

impl MeshInstance {
    pub fn create(shared_data: &SharedDataRw, mesh_data: MeshData) -> MeshId {
        let mut mesh_instance = MeshInstance::default();
        let mut data = shared_data.write().unwrap();
        mesh_instance.mesh_data = mesh_data;
        data.add_resource(mesh_instance)
    }
    pub fn get_data(&self) -> &MeshData {
        &self.mesh_data
    }
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
    pub fn set_visible(shared_data: &SharedDataRw, mesh_id: MeshId, is_visible: bool) {
        let mesh = SharedData::get_resource::<Self>(shared_data, mesh_id);
        let mesh = &mut mesh.get_mut();
        mesh.is_visible = is_visible;
        mesh.is_dirty = true;
    }
    pub fn set_draw_area(shared_data: &SharedDataRw, mesh_id: MeshId, draw_area: Vector4) {
        let mesh = SharedData::get_resource::<Self>(shared_data, mesh_id);
        let mesh = &mut mesh.get_mut();
        mesh.draw_area = draw_area;
        mesh.is_dirty = true;
    }
    pub fn set_transform(shared_data: &SharedDataRw, mesh_id: MeshId, transform: Matrix4) {
        let mesh = SharedData::get_resource::<Self>(shared_data, mesh_id);
        let mesh = &mut mesh.get_mut();
        mesh.transform = transform;
        mesh.is_dirty = true;
    }
    pub fn set_mesh_data(shared_data: &SharedDataRw, mesh_id: MeshId, mesh_data: MeshData) {
        let mesh = SharedData::get_resource::<Self>(shared_data, mesh_id);
        let mesh = &mut mesh.get_mut();
        mesh.mesh_data = mesh_data;
        mesh.uv_converted = false;
        mesh.is_dirty = true;
    }
    pub fn get_transform(&self) -> &Matrix4 {
        &self.transform
    }
    pub fn get_draw_area(&self) -> Vector4 {
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

    pub fn destroy(shared_data: &SharedDataRw, mesh_id: MeshId) {
        let mut data = shared_data.write().unwrap();
        data.remove_resource::<Self>(mesh_id)
    }
}
