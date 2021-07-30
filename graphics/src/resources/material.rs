use std::path::{Path, PathBuf};

use crate::{
    MaterialData, MeshId, MeshInstance, MeshRc, PipelineInstance, PipelineRc, TextureId,
    TextureInstance, TextureRc,
};
use nrg_math::{VecBase, Vector4};
use nrg_resources::{
    DataTypeResource, Deserializable, FileResource, ResourceData, ResourceId, ResourceRef,
    SerializableResource, SharedData, SharedDataRw,
};
use nrg_serialize::{generate_random_uid, generate_uid_from_string, INVALID_UID};

pub type MaterialId = ResourceId;
pub type MaterialRc = ResourceRef<MaterialInstance>;

pub struct MaterialInstance {
    id: ResourceId,
    path: PathBuf,
    pipeline: PipelineRc,
    meshes: Vec<MeshRc>,
    textures: Vec<TextureRc>,
    diffuse_color: Vector4,
    outline_color: Vector4,
}

impl Default for MaterialInstance {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            path: PathBuf::new(),
            pipeline: ResourceRef::default(),
            meshes: Vec::new(),
            textures: Vec::new(),
            diffuse_color: [1., 1., 1., 1.].into(),
            outline_color: Vector4::default_zero(),
        }
    }
}

impl ResourceData for MaterialInstance {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl SerializableResource for MaterialInstance {
    fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl DataTypeResource for MaterialInstance {
    type DataType = MaterialData;

    fn create_from_data(shared_data: &SharedDataRw, material_data: Self::DataType) -> MaterialRc {
        if let Some(pipeline) =
            PipelineInstance::find_from_name(shared_data, material_data.pipeline_name.as_str())
        {
            let mut meshes = Vec::new();
            for m in material_data.meshes.iter() {
                let mesh = MeshInstance::create_from_file(&shared_data, m.as_path());
                meshes.push(mesh);
            }
            let mut textures = Vec::new();
            for t in material_data.textures.iter() {
                let texture = TextureInstance::create_from_file(&shared_data, t.as_path());
                textures.push(texture);
            }

            let material = Self {
                id: generate_uid_from_string(material_data.path().to_str().unwrap()),
                path: material_data.path().to_path_buf(),
                pipeline,
                meshes,
                textures,
                ..Default::default()
            };

            SharedData::add_resource(shared_data, material)
        } else {
            panic!(
                "No pipeline loaded with name {}",
                material_data.pipeline_name.as_str()
            );
        }
    }
}

impl MaterialInstance {
    pub fn find_from_path(shared_data: &SharedDataRw, path: &Path) -> Option<MaterialRc> {
        SharedData::match_resource(shared_data, |m: &MaterialInstance| m.path() == path)
    }
    pub fn pipeline(&self) -> PipelineRc {
        self.pipeline.clone()
    }
    pub fn has_meshes(&self) -> bool {
        !self.meshes.is_empty()
    }
    pub fn meshes(&self) -> &Vec<MeshRc> {
        &self.meshes
    }
    pub fn textures(&self) -> &Vec<TextureRc> {
        &self.textures
    }
    pub fn has_diffuse_texture(&self) -> bool {
        !self.textures.is_empty()
    }
    pub fn diffuse_texture(&self) -> TextureRc {
        self.textures[0].clone()
    }
    pub fn diffuse_color(&self) -> Vector4 {
        self.diffuse_color
    }
    pub fn outline_color(&self) -> Vector4 {
        self.outline_color
    }
    pub fn has_textures(&self) -> bool {
        !self.textures.is_empty()
    }

    pub fn has_texture(&self, texture_id: TextureId) -> bool {
        self.textures.iter().any(|t| t.id() == texture_id)
    }

    pub fn remove_texture(&mut self, texture_id: TextureId) {
        self.textures.retain(|t| t.id() != texture_id);
    }
    pub fn remove_all_textures(&mut self) -> &mut Self {
        self.textures.clear();
        self
    }

    pub fn add_texture(&mut self, texture: TextureRc) -> &mut Self {
        self.textures.push(texture);
        self
    }

    pub fn add_mesh(&mut self, mesh: MeshRc) -> &mut Self {
        self.meshes.push(mesh);
        self
    }

    pub fn remove_mesh(&mut self, mesh_id: MeshId) {
        self.meshes.retain(|m| m.id() != mesh_id);
    }
    pub fn remove_all_meshes(&mut self) -> &mut Self {
        self.meshes.clear();
        self
    }

    pub fn set_diffuse_color(&mut self, diffuse_color: Vector4) {
        self.diffuse_color = diffuse_color;
    }

    pub fn set_outline_color(&mut self, outline_color: Vector4) {
        self.outline_color = outline_color;
    }

    pub fn create_from_pipeline(shared_data: &SharedDataRw, pipeline: PipelineRc) -> MaterialRc {
        SharedData::add_resource(
            shared_data,
            MaterialInstance {
                id: generate_random_uid(),
                pipeline,
                ..Default::default()
            },
        )
    }
}
