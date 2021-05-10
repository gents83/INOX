use crate::{MeshData, Texture};
use nrg_math::{MatBase, Matrix4};
use nrg_resources::{ResourceId, SharedData, SharedDataRw};

pub type MeshId = ResourceId;

#[derive(Clone)]
pub struct MeshInstance {
    mesh_data: MeshData,
    transform: Matrix4,
    is_visible: bool,
    is_dirty: bool,
    uv_converted: bool,
}

impl MeshInstance {
    pub fn create(shared_data: &SharedDataRw, mesh_data: MeshData) -> MeshId {
        let mut data = shared_data.write().unwrap();
        data.add_resource(MeshInstance {
            mesh_data,
            transform: Matrix4::default_identity(),
            is_visible: true,
            is_dirty: true,
            uv_converted: false,
        })
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
    pub fn set_transform(shared_data: &SharedDataRw, mesh_id: MeshId, transform: Matrix4) {
        let mesh = SharedData::get_resource::<Self>(shared_data, mesh_id);
        let mesh = &mut mesh.get_mut();
        mesh.transform = transform;
        mesh.is_dirty = true;
    }
    pub fn get_transform(&self) -> Matrix4 {
        self.transform
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
