use std::path::{Path, PathBuf};

use crate::{MaterialData, Pipeline, Texture, TextureId};

use nrg_math::{VecBase, Vector4};
use nrg_resources::{
    DataTypeResource, Deserializable, FileResource, Handle, Resource, ResourceData, ResourceId,
    SerializableResource, SharedData, SharedDataRw,
};
use nrg_serialize::{generate_random_uid, generate_uid_from_string, INVALID_UID};

pub type MaterialId = ResourceId;

pub struct Material {
    id: ResourceId,
    pipeline: Handle<Pipeline>,
    path: PathBuf,
    textures: Vec<Resource<Texture>>,
    diffuse_color: Vector4,
    outline_color: Vector4,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            pipeline: None,
            path: PathBuf::new(),
            textures: Vec::new(),
            diffuse_color: [1., 1., 1., 1.].into(),
            outline_color: Vector4::default_zero(),
        }
    }
}

impl ResourceData for Material {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl SerializableResource for Material {
    fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl DataTypeResource for Material {
    type DataType = MaterialData;

    fn create_from_data(
        shared_data: &SharedDataRw,
        material_data: Self::DataType,
    ) -> Resource<Self> {
        let id = generate_uid_from_string(material_data.path().to_str().unwrap());
        let path = material_data.path().to_path_buf();

        let mut textures = Vec::new();
        for t in material_data.textures.iter() {
            let texture = Texture::create_from_file(shared_data, t.as_path());
            textures.push(texture);
        }

        let pipeline = Pipeline::create_from_data(shared_data, material_data.pipeline);

        let material = Self {
            id,
            path,
            textures,
            pipeline: Some(pipeline),
            diffuse_color: [1., 1., 1., 1.].into(),
            outline_color: Vector4::default_zero(),
        };

        SharedData::add_resource(shared_data, material)
    }
}

impl Material {
    pub fn create_from_pipeline(
        shared_data: &SharedDataRw,
        pipeline: &Resource<Pipeline>,
    ) -> Resource<Self> {
        SharedData::add_resource(
            shared_data,
            Self {
                id: generate_random_uid(),
                pipeline: Some(pipeline.clone()),
                path: PathBuf::new(),
                textures: Vec::new(),
                diffuse_color: [1., 1., 1., 1.].into(),
                outline_color: Vector4::default_zero(),
            },
        )
    }
    pub fn find_from_path(shared_data: &SharedDataRw, path: &Path) -> Handle<Material> {
        SharedData::match_resource(shared_data, |m: &Material| m.path() == path)
    }
    pub fn pipeline(&self) -> &Handle<Pipeline> {
        &self.pipeline
    }
    pub fn textures(&self) -> &Vec<Resource<Texture>> {
        &self.textures
    }
    pub fn has_diffuse_texture(&self) -> bool {
        !self.textures.is_empty()
    }
    pub fn diffuse_texture(&self) -> &Resource<Texture> {
        &self.textures[0]
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

    pub fn add_texture(&mut self, texture: Resource<Texture>) -> &mut Self {
        self.textures.push(texture);
        self
    }
    pub fn set_diffuse_color(&mut self, diffuse_color: Vector4) {
        self.diffuse_color = diffuse_color;
    }

    pub fn set_outline_color(&mut self, outline_color: Vector4) {
        self.outline_color = outline_color;
    }
}
