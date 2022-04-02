use std::path::{Path, PathBuf};

use crate::{
    MaterialAlphaMode, MaterialData, Pipeline, Texture, TextureId, TextureType, INVALID_INDEX,
};

use inox_math::Vector4;
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceEvent, ResourceId, ResourceTrait,
    SerializableResource, SharedData, SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file, SerializeFile};
use inox_uid::generate_random_uid;

pub type MaterialId = ResourceId;

#[derive(Clone)]
pub struct Material {
    id: MaterialId,
    message_hub: MessageHubRc,
    shared_data: SharedDataRc,
    pipeline: Handle<Pipeline>,
    uniform_index: i32,
    filepath: PathBuf,
    textures: [Handle<Texture>; TextureType::Count as _], // use specular glossiness if specular_glossiness_texture set
    textures_coords_set: [u32; TextureType::Count as _],
    roughness_factor: f32,
    metallic_factor: f32,
    alpha_cutoff: f32,
    alpha_mode: MaterialAlphaMode,
    base_color: Vector4,
    emissive_color: Vector4,
    diffuse_color: Vector4,
    specular_color: Vector4,
}

impl ResourceTrait for Material {
    type OnCreateData = ();

    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &MaterialId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(
        &mut self,
        _shared_data: &SharedData,
        _message_hub: &MessageHubRc,
        _id: &MaterialId,
    ) {
    }
    fn on_copy(&mut self, other: &Self)
    where
        Self: Sized,
    {
        *self = other.clone();
    }
}

impl SerializableResource for Material {
    fn set_path(&mut self, path: &Path) -> &mut Self {
        self.filepath = path.to_path_buf();
        self
    }
    fn path(&self) -> &Path {
        self.filepath.as_path()
    }

    fn extension() -> &'static str {
        MaterialData::extension()
    }
}

impl DataTypeResource for Material {
    type DataType = MaterialData;
    type OnCreateData = <Self as ResourceTrait>::OnCreateData;

    fn new(id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            id,
            message_hub: message_hub.clone(),
            shared_data: shared_data.clone(),
            pipeline: None,
            uniform_index: INVALID_INDEX,
            filepath: PathBuf::new(),
            textures: Default::default(),
            textures_coords_set: Default::default(),
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
    fn is_initialized(&self) -> bool {
        self.uniform_index != INVALID_INDEX
    }
    fn invalidate(&mut self) -> &mut Self {
        self.uniform_index = INVALID_INDEX;
        self
    }
    fn deserialize_data(
        path: &std::path::Path,
        registry: &SerializableRegistryRc,
        f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
        read_from_file::<Self::DataType>(path, registry, f);
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: ResourceId,
        material_data: Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let mut textures: [Handle<Texture>; TextureType::Count as _] = Default::default();
        for (i, t) in material_data.textures.iter().enumerate() {
            if !t.as_os_str().is_empty() {
                let texture = Texture::request_load(shared_data, message_hub, t.as_path(), None);
                textures[i] = Some(texture);
            }
        }
        let mut textures_coords_set: [u32; TextureType::Count as _] = Default::default();
        for (i, t) in material_data.texcoords_set.iter().enumerate() {
            textures_coords_set[i] = *t as _;
        }

        let pipeline = if material_data.pipeline.as_os_str().is_empty() {
            None
        } else {
            Some(Pipeline::request_load(
                shared_data,
                message_hub,
                material_data.pipeline.as_path(),
                None,
            ))
        };

        Self {
            id,
            message_hub: message_hub.clone(),
            shared_data: shared_data.clone(),
            textures,
            textures_coords_set,
            roughness_factor: material_data.roughness_factor,
            metallic_factor: material_data.metallic_factor,
            alpha_cutoff: material_data.alpha_cutoff,
            alpha_mode: material_data.alpha_mode,
            base_color: material_data.base_color,
            emissive_color: material_data.emissive_color,
            diffuse_color: material_data.diffuse_color,
            specular_color: material_data.specular_color,
            pipeline,
            uniform_index: INVALID_INDEX,
            filepath: PathBuf::new(),
        }
    }
}

impl Material {
    pub fn mark_as_dirty(&self) -> &Self {
        self.message_hub
            .send_event(ResourceEvent::<Self>::Changed(self.id));
        self
    }
    pub fn duplicate_from_pipeline(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        pipeline: &Resource<Pipeline>,
    ) -> Resource<Self> {
        let id = generate_random_uid();
        let mut data = Self::new(id, shared_data, message_hub);
        data.pipeline = Some(pipeline.clone());
        SharedData::add_resource(shared_data, message_hub, data.id, data)
    }
    pub fn pipeline(&self) -> &Handle<Pipeline> {
        &self.pipeline
    }
    pub fn uniform_index(&self) -> i32 {
        self.uniform_index
    }

    pub fn update_uniform(&mut self, uniform_index: u32) -> bool {
        let mut is_changed = false;
        if self.uniform_index != uniform_index as i32 {
            is_changed = true;
            self.uniform_index = uniform_index as _;
        }
        is_changed
    }

    pub fn textures(&self) -> &[Handle<Texture>; TextureType::Count as _] {
        &self.textures
    }
    pub fn textures_coords_set(&self) -> &[u32; TextureType::Count as _] {
        &self.textures_coords_set
    }
    pub fn has_texture_id(&self, texture_id: &TextureId) -> bool {
        let mut has_texture = false;
        self.textures.iter().for_each(|t| {
            if let Some(texture) = t {
                if texture.id() == texture_id {
                    has_texture = true;
                }
            }
        });
        has_texture
    }
    pub fn has_texture(&self, texture_type: TextureType) -> bool {
        self.textures[texture_type as usize].is_some()
    }
    pub fn texture(&self, texture_type: TextureType) -> &Handle<Texture> {
        &self.textures[texture_type as usize]
    }

    pub fn remove_texture(&mut self, texture_type: TextureType) -> &mut Self {
        self.textures[texture_type as usize] = None;
        self.mark_as_dirty();
        self
    }
    pub fn set_texture(
        &mut self,
        texture_type: TextureType,
        texture: &Resource<Texture>,
    ) -> &mut Self {
        self.textures[texture_type as usize] = Some(texture.clone());
        self.mark_as_dirty();
        self
    }

    pub fn roughness_factor(&self) -> f32 {
        self.roughness_factor
    }
    pub fn set_roughness_factor(&mut self, roughness_factor: f32) {
        self.roughness_factor = roughness_factor;
        self.mark_as_dirty();
    }
    pub fn metallic_factor(&self) -> f32 {
        self.metallic_factor
    }
    pub fn set_metallic_factor(&mut self, metallic_factor: f32) {
        self.metallic_factor = metallic_factor;
        self.mark_as_dirty();
    }
    pub fn alpha_cutoff(&self) -> f32 {
        self.alpha_cutoff
    }
    pub fn set_alpha_cutoff(&mut self, alpha_cutoff: f32) {
        self.alpha_cutoff = alpha_cutoff;
        self.mark_as_dirty();
    }

    pub fn alpha_mode(&self) -> MaterialAlphaMode {
        self.alpha_mode
    }
    pub fn set_alpha_mode(&mut self, alpha_mode: MaterialAlphaMode) {
        self.alpha_mode = alpha_mode;
        self.mark_as_dirty();
    }

    pub fn base_color(&self) -> Vector4 {
        self.base_color
    }
    pub fn set_base_color(&mut self, base_color: Vector4) {
        self.base_color = base_color;
        self.mark_as_dirty();
    }
    pub fn emissive_color(&self) -> Vector4 {
        self.emissive_color
    }
    pub fn set_emissive_color(&mut self, emissive_color: Vector4) {
        self.emissive_color = emissive_color;
        self.mark_as_dirty();
    }
    pub fn diffuse_color(&self) -> Vector4 {
        self.diffuse_color
    }
    pub fn set_diffuse_color(&mut self, diffuse_color: Vector4) {
        self.diffuse_color = diffuse_color;
        self.mark_as_dirty();
    }
    pub fn specular_color(&self) -> Vector4 {
        self.specular_color
    }
    pub fn set_specular_color(&mut self, specular_color: Vector4) {
        self.specular_color = specular_color;
        self.mark_as_dirty();
    }
}
