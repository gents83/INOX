use crate::{FontId, FontInstance, MeshId, PipelineId, TextureId};
use nrg_math::Vector4;
use nrg_resources::{ResourceId, SharedDataRw};
use nrg_serialize::INVALID_UID;

pub type MaterialId = ResourceId;

pub struct MaterialInstance {
    pipeline_id: PipelineId,
    meshes: Vec<MeshId>,
    textures: Vec<TextureId>,
    diffuse_color: Vector4,
}

impl MaterialInstance {
    pub fn create_from(shared_data: &SharedDataRw, material_id: MaterialId) -> Self {
        let data = shared_data.read().unwrap();
        let material = data.get_resource::<MaterialInstance>(material_id);
        let pipeline_id = material.get().pipeline_id;
        let textures = material.get().textures.clone();
        let diffuse_color = material.get().diffuse_color;
        Self {
            pipeline_id,
            meshes: Vec::new(),
            textures,
            diffuse_color,
        }
    }
    pub fn get_pipeline_id(&self) -> PipelineId {
        self.pipeline_id
    }
    pub fn has_meshes(&self) -> bool {
        !self.meshes.is_empty()
    }
    pub fn get_meshes(&self) -> &Vec<MeshId> {
        &self.meshes
    }
    pub fn get_textures(&self) -> &Vec<TextureId> {
        &self.textures
    }
    pub fn get_diffuse_texture(&self) -> TextureId {
        if !self.textures.is_empty() {
            return self.textures[0];
        }
        INVALID_UID
    }
    pub fn get_diffuse_color(&self) -> Vector4 {
        self.diffuse_color
    }

    pub fn add_texture(shared_data: &SharedDataRw, material_id: MaterialId, texture_id: TextureId) {
        let data = shared_data.read().unwrap();
        let material = data.get_resource::<Self>(material_id);
        material.get_mut().textures.push(texture_id);
    }

    pub fn add_mesh(shared_data: &SharedDataRw, material_id: MaterialId, mesh_id: MeshId) {
        let data = shared_data.read().unwrap();
        let material = data.get_resource::<Self>(material_id);
        material.get_mut().meshes.push(mesh_id);
    }

    pub fn remove_mesh(shared_data: &SharedDataRw, material_id: MaterialId, mesh_id: MeshId) {
        let data = shared_data.read().unwrap();
        let material = data.get_resource::<Self>(material_id);
        let meshes = &mut material.get_mut().meshes;
        if let Some(index) = meshes.iter().position(|&id| id == mesh_id) {
            meshes.remove(index);
        }
    }

    pub fn set_diffuse_color(
        shared_data: &SharedDataRw,
        material_id: MaterialId,
        diffuse_color: Vector4,
    ) {
        let data = shared_data.read().unwrap();
        let material = data.get_resource::<Self>(material_id);
        material.get_mut().diffuse_color = diffuse_color;
    }

    pub fn create_from_pipeline(shared_data: &SharedDataRw, pipeline_id: PipelineId) -> MaterialId {
        let mut data = shared_data.write().unwrap();
        data.add_resource(MaterialInstance {
            pipeline_id,
            meshes: Vec::new(),
            textures: Vec::new(),
            diffuse_color: [1., 1., 1., 1.].into(),
        })
    }

    pub fn create_from_font(shared_data: &SharedDataRw, font_id: FontId) -> MaterialId {
        let material_id = FontInstance::get_material(shared_data, font_id);
        let material = MaterialInstance::create_from(shared_data, material_id);
        let mut data = shared_data.write().unwrap();
        data.add_resource(material)
    }

    pub fn destroy(shared_data: &SharedDataRw, material_id: MaterialId) {
        let mut data = shared_data.write().unwrap();
        data.remove_resource::<Self>(material_id)
    }
}
