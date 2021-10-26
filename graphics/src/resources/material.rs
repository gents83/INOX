use std::path::{Path, PathBuf};

use crate::{MaterialData, Pipeline, Texture, TextureId};

use nrg_math::{VecBase, Vector4};
use nrg_messenger::MessengerRw;
use nrg_resources::{
    DataTypeResource, Handle, Resource, ResourceId, SerializableResource, SharedData, SharedDataRc,
};
use nrg_serialize::{generate_random_uid, read_from_file};

pub type MaterialId = ResourceId;

#[derive(Clone)]
pub struct Material {
    pipeline: Handle<Pipeline>,
    path: PathBuf,
    textures: Vec<Resource<Texture>>,
    diffuse_color: Vector4,
    outline_color: Vector4,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            pipeline: None,
            path: PathBuf::new(),
            textures: Vec::new(),
            diffuse_color: [1., 1., 1., 1.].into(),
            outline_color: Vector4::default_zero(),
        }
    }
}

impl SerializableResource for Material {
    fn set_path(&mut self, path: &Path) {
        self.path = path.to_path_buf();
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }
    fn is_matching_extension(path: &Path) -> bool {
        const MATERIAL_EXTENSION: &str = "material_data";

        if let Some(ext) = path.extension().unwrap().to_str() {
            return ext == MATERIAL_EXTENSION;
        }
        false
    }
}

impl DataTypeResource for Material {
    type DataType = MaterialData;

    fn is_initialized(&self) -> bool {
        self.pipeline.is_some()
    }
    fn invalidate(&mut self) {
        self.pipeline = None;
    }
    fn deserialize_data(path: &Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        id: MaterialId,
        material_data: Self::DataType,
    ) -> Resource<Self> {
        let mut textures = Vec::new();
        for t in material_data.textures.iter() {
            let texture = Texture::load_from_file(shared_data, global_messenger, t.as_path(), None);
            textures.push(texture);
        }

        let pipeline = Pipeline::load_from_file(
            shared_data,
            global_messenger,
            material_data.pipeline.as_path(),
            None,
        );

        let material = Self {
            textures,
            diffuse_color: material_data.diffuse_color,
            outline_color: material_data.outline_color,
            pipeline: Some(pipeline),
            ..Default::default()
        };
        SharedData::add_resource(shared_data, id, material)
    }
}

impl Material {
    pub fn duplicate_from_pipeline(
        shared_data: &SharedDataRc,
        pipeline: &Resource<Pipeline>,
    ) -> Resource<Self> {
        SharedData::add_resource(
            shared_data,
            generate_random_uid(),
            Self {
                pipeline: Some(pipeline.clone()),
                path: PathBuf::new(),
                textures: Vec::new(),
                diffuse_color: [1., 1., 1., 1.].into(),
                outline_color: Vector4::default_zero(),
            },
        )
    }
    pub fn find_from_path(shared_data: &SharedDataRc, path: &Path) -> Handle<Material> {
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

    pub fn has_texture(&self, texture_id: &TextureId) -> bool {
        self.textures.iter().any(|t| t.id() == texture_id)
    }

    pub fn remove_texture(&mut self, texture_id: &TextureId) {
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
