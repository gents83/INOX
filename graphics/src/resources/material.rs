use std::path::{Path, PathBuf};

use crate::{
    MaterialAlphaMode, MaterialData, Pipeline, ShaderMaterialData, Texture, TextureType,
    INVALID_INDEX,
};

use nrg_math::Vector4;
use nrg_messenger::MessengerRw;
use nrg_resources::{
    DataTypeResource, Handle, Resource, ResourceId, SerializableResource, SharedData, SharedDataRc,
};
use nrg_serialize::{generate_random_uid, read_from_file};

pub type MaterialId = ResourceId;

#[derive(Clone)]
pub struct Material {
    pipeline: Handle<Pipeline>,
    uniform_index: i32,
    filepath: PathBuf,
    textures: [Handle<Texture>; TextureType::Count as _], // use specular glossiness if specular_glossiness_texture set
    roughness_factor: f32,
    metallic_factor: f32,
    alpha_cutoff: f32,
    alpha_mode: MaterialAlphaMode,
    base_color: Vector4,
    emissive_color: Vector4,
    diffuse_color: Vector4,
    specular_color: Vector4,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            pipeline: None,
            uniform_index: INVALID_INDEX,
            filepath: PathBuf::new(),
            textures: Default::default(),
            roughness_factor: 1.,
            metallic_factor: 1.,
            alpha_cutoff: 1.,
            alpha_mode: MaterialAlphaMode::Opaque,
            base_color: Vector4::new(1., 1., 1., 1.),
            emissive_color: Vector4::new(1., 1., 1., 1.),
            diffuse_color: Vector4::new(1., 1., 1., 1.),
            specular_color: Vector4::new(0., 0., 0., 1.),
        }
    }
}

impl SerializableResource for Material {
    fn set_path(&mut self, path: &Path) {
        self.filepath = path.to_path_buf();
    }
    fn path(&self) -> &Path {
        self.filepath.as_path()
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
        self.uniform_index != INVALID_INDEX
    }
    fn invalidate(&mut self) {
        self.uniform_index = INVALID_INDEX;
    }
    fn deserialize_data(path: &Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        _id: ResourceId,
        material_data: Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let mut textures: [Handle<Texture>; TextureType::Count as _] = Default::default();
        for (i, t) in material_data.textures.iter().enumerate() {
            if !t.as_os_str().is_empty() {
                let texture =
                    Texture::load_from_file(shared_data, global_messenger, t.as_path(), None);
                textures[i] = Some(texture);
            }
        }

        let pipeline = if material_data.pipeline.as_os_str().is_empty() {
            None
        } else {
            Some(Pipeline::load_from_file(
                shared_data,
                global_messenger,
                material_data.pipeline.as_path(),
                None,
            ))
        };

        Self {
            textures,
            roughness_factor: material_data.roughness_factor,
            metallic_factor: material_data.metallic_factor,
            alpha_cutoff: material_data.alpha_cutoff,
            alpha_mode: material_data.alpha_mode,
            base_color: material_data.base_color,
            emissive_color: material_data.emissive_color,
            diffuse_color: material_data.diffuse_color,
            specular_color: material_data.specular_color,
            pipeline,
            ..Default::default()
        }
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
                ..Default::default()
            },
        )
    }
    pub fn pipeline(&self) -> &Handle<Pipeline> {
        &self.pipeline
    }
    pub fn uniform_index(&self) -> i32 {
        self.uniform_index
    }
    pub fn set_uniform_index(&mut self, uniform_index: i32) {
        self.uniform_index = uniform_index;
    }

    pub fn textures(&self) -> &[Handle<Texture>; TextureType::Count as _] {
        &self.textures
    }
    pub fn has_texture(&self, texture_type: TextureType) -> bool {
        self.textures[texture_type as usize].is_some()
    }
    pub fn texture(&self, texture_type: TextureType) -> &Handle<Texture> {
        &self.textures[texture_type as usize]
    }
    pub fn remove_texture(&mut self, texture_type: TextureType) {
        self.textures[texture_type as usize] = None;
    }
    pub fn set_texture(
        &mut self,
        texture_type: TextureType,
        texture: &Resource<Texture>,
    ) -> &mut Self {
        self.textures[texture_type as usize] = Some(texture.clone());
        self
    }

    pub fn roughness_factor(&self) -> f32 {
        self.roughness_factor
    }
    pub fn set_roughness_factor(&mut self, roughness_factor: f32) {
        self.roughness_factor = roughness_factor;
    }
    pub fn metallic_factor(&self) -> f32 {
        self.metallic_factor
    }
    pub fn set_metallic_factor(&mut self, metallic_factor: f32) {
        self.metallic_factor = metallic_factor;
    }
    pub fn alpha_cutoff(&self) -> f32 {
        self.alpha_cutoff
    }
    pub fn set_alpha_cutoff(&mut self, alpha_cutoff: f32) {
        self.alpha_cutoff = alpha_cutoff;
    }

    pub fn alpha_mode(&self) -> MaterialAlphaMode {
        self.alpha_mode
    }
    pub fn set_alpha_mode(&mut self, alpha_mode: MaterialAlphaMode) {
        self.alpha_mode = alpha_mode;
    }

    pub fn base_color(&self) -> Vector4 {
        self.base_color
    }
    pub fn set_base_color(&mut self, base_color: Vector4) {
        self.base_color = base_color;
    }
    pub fn emissive_color(&self) -> Vector4 {
        self.emissive_color
    }
    pub fn set_emissive_color(&mut self, emissive_color: Vector4) {
        self.emissive_color = emissive_color;
    }
    pub fn diffuse_color(&self) -> Vector4 {
        self.diffuse_color
    }
    pub fn set_diffuse_color(&mut self, diffuse_color: Vector4) {
        self.diffuse_color = diffuse_color;
    }
    pub fn specular_color(&self) -> Vector4 {
        self.specular_color
    }
    pub fn set_specular_color(&mut self, specular_color: Vector4) {
        self.specular_color = specular_color;
    }

    pub fn create_uniform_material_data(&self) -> ShaderMaterialData {
        let mut textures_indices = [INVALID_INDEX; TextureType::Count as _];
        for i in 0..TextureType::Count as usize {
            if let Some(texture) = &self.textures[i] {
                texture.get(|t| {
                    textures_indices[i] = t.texture_index();
                });
            }
        }
        ShaderMaterialData {
            textures_indices,
            roughness_factor: self.roughness_factor,
            metallic_factor: self.metallic_factor,
            alpha_cutoff: self.alpha_cutoff,
            alpha_mode: self.alpha_mode as _,
            base_color: self.base_color.into(),
            emissive_color: self.emissive_color.into(),
            diffuse_color: self.diffuse_color.into(),
            specular_color: self.specular_color.into(),
            ..Default::default()
        }
    }
}
