use std::path::{Path, PathBuf};

use crate::{
    FontId, FontInstance, MaterialData, MeshId, MeshInstance, PipelineId, PipelineInstance,
    TextureId, TextureInstance,
};
use nrg_math::Vector4;
use nrg_resources::{from_file, ResourceId, ResourceTrait, SharedData, SharedDataRw};
use nrg_serialize::{generate_random_uid, generate_uid_from_string, INVALID_UID};

pub type MaterialId = ResourceId;

pub struct MaterialInstance {
    id: ResourceId,
    pipeline_id: PipelineId,
    meshes: Vec<MeshId>,
    textures: Vec<TextureId>,
    diffuse_color: Vector4,
    outline_color: Vector4,
}

impl ResourceTrait for MaterialInstance {
    fn id(&self) -> ResourceId {
        self.id
    }
    fn path(&self) -> PathBuf {
        PathBuf::default()
    }
}

impl MaterialInstance {
    pub fn create_from_file(shared_data: &SharedDataRw, filepath: &Path) -> MaterialId {
        let material_data = from_file::<MaterialData>(filepath);

        let pipeline_id =
            PipelineInstance::find_id_from_name(shared_data, material_data.pipeline_name.as_str());

        let mut meshes = Vec::new();
        for m in material_data.meshes.iter() {
            let mesh_id = MeshInstance::create_from_file(&shared_data, m.as_path());
            meshes.push(mesh_id);
        }
        let mut textures = Vec::new();
        for t in material_data.textures.iter() {
            let texture_id = TextureInstance::create_from_file(&shared_data, t.as_path());
            textures.push(texture_id);
        }

        let material = Self {
            id: generate_uid_from_string(filepath.to_str().unwrap()),
            pipeline_id,
            meshes,
            textures,
            diffuse_color: [1., 1., 1., 1.].into(),
            outline_color: [1., 1., 1., 0.].into(),
        };

        let mut data = shared_data.write().unwrap();
        data.add_resource(material)
    }
    pub fn create_from(shared_data: &SharedDataRw, material_id: MaterialId) -> Self {
        let material = SharedData::get_resource::<MaterialInstance>(shared_data, material_id);
        let pipeline_id = material.get().pipeline_id;
        let textures = material.get().textures.clone();
        let diffuse_color = material.get().diffuse_color;
        let outline_color = material.get().outline_color;
        Self {
            id: generate_random_uid(),
            pipeline_id,
            meshes: Vec::new(),
            textures,
            diffuse_color,
            outline_color,
        }
    }
    pub fn get_pipeline_id(&self) -> PipelineId {
        self.pipeline_id
    }
    pub fn has_meshes(&self) -> bool {
        !self.meshes.is_empty()
    }
    pub fn meshes(&self) -> &Vec<MeshId> {
        &self.meshes
    }
    pub fn textures(&self) -> &Vec<TextureId> {
        &self.textures
    }
    pub fn diffuse_texture(&self) -> TextureId {
        if !self.textures.is_empty() {
            return self.textures[0];
        }
        INVALID_UID
    }
    pub fn diffuse_color(&self) -> Vector4 {
        self.diffuse_color
    }
    pub fn outline_color(&self) -> Vector4 {
        self.outline_color
    }
    pub fn get_meshes(shared_data: &SharedDataRw, material_id: MaterialId) -> Vec<MeshId> {
        let material = SharedData::get_resource::<Self>(shared_data, material_id);
        let material = material.get();
        material.meshes.clone()
    }
    pub fn has_textures(shared_data: &SharedDataRw, material_id: MaterialId) -> bool {
        let material = SharedData::get_resource::<Self>(shared_data, material_id);
        let textures = &material.get().textures;
        !textures.is_empty()
    }

    pub fn has_texture(
        shared_data: &SharedDataRw,
        material_id: MaterialId,
        texture_id: TextureId,
    ) -> bool {
        let material = SharedData::get_resource::<Self>(shared_data, material_id);
        let textures = &material.get().textures;
        textures.iter().any(|&id| id == texture_id)
    }

    pub fn remove_texture(
        shared_data: &SharedDataRw,
        material_id: MaterialId,
        texture_id: TextureId,
    ) {
        let material = SharedData::get_resource::<Self>(shared_data, material_id);
        material.get_mut().textures.retain(|&id| id != texture_id);
    }

    pub fn add_texture(shared_data: &SharedDataRw, material_id: MaterialId, texture_id: TextureId) {
        let material = SharedData::get_resource::<Self>(shared_data, material_id);
        material.get_mut().textures.push(texture_id);
    }

    pub fn add_mesh(shared_data: &SharedDataRw, material_id: MaterialId, mesh_id: MeshId) {
        let material = SharedData::get_resource::<Self>(shared_data, material_id);
        material.get_mut().meshes.push(mesh_id);
    }

    pub fn remove_mesh(shared_data: &SharedDataRw, material_id: MaterialId, mesh_id: MeshId) {
        let material = SharedData::get_resource::<Self>(shared_data, material_id);
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
        let material = SharedData::get_resource::<Self>(shared_data, material_id);
        material.get_mut().diffuse_color = diffuse_color;
    }

    pub fn set_outline_color(
        shared_data: &SharedDataRw,
        material_id: MaterialId,
        outline_color: Vector4,
    ) {
        let material = SharedData::get_resource::<Self>(shared_data, material_id);
        material.get_mut().outline_color = outline_color;
    }

    pub fn create_from_pipeline(shared_data: &SharedDataRw, pipeline_id: PipelineId) -> MaterialId {
        let mut data = shared_data.write().unwrap();
        data.add_resource(MaterialInstance {
            id: generate_random_uid(),
            pipeline_id,
            meshes: Vec::new(),
            textures: Vec::new(),
            diffuse_color: [1., 1., 1., 1.].into(),
            outline_color: [1., 1., 1., 0.].into(),
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
