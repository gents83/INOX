use std::path::{Path, PathBuf};

use crate::{MaterialData, Texture, TextureId, TextureType, INVALID_INDEX};

use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceEvent, ResourceId, ResourceTrait,
    SerializableResource, SharedDataRc,
};
use inox_serialize::{
    inox_serializable::SerializableRegistryRc, read_from_file, SerializationType, SerializeFile,
};

pub type MaterialId = ResourceId;

#[derive(Clone)]
pub struct Material {
    id: MaterialId,
    path: PathBuf,
    message_hub: MessageHubRc,
    shared_data: SharedDataRc,
    textures: [Handle<Texture>; TextureType::Count as _],
    material_index: i32,
}

impl ResourceTrait for Material {
    fn is_initialized(&self) -> bool {
        self.material_index != INVALID_INDEX
    }
    fn invalidate(&mut self) -> &mut Self {
        self.material_index = INVALID_INDEX;
        self
    }
}

impl SerializableResource for Material {
    fn set_path(&mut self, path: &Path) -> &mut Self {
        self.path = path.to_path_buf();
        self
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn extension() -> &'static str {
        MaterialData::extension()
    }

    fn deserialize_data(
        path: &std::path::Path,
        registry: SerializableRegistryRc,
        f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
        read_from_file::<Self::DataType>(path, registry, SerializationType::Binary, f);
    }
}

impl DataTypeResource for Material {
    type DataType = MaterialData;

    fn new(id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            id,
            message_hub: message_hub.clone(),
            shared_data: shared_data.clone(),
            material_index: INVALID_INDEX,
            path: PathBuf::new(),
            textures: Default::default(),
        }
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: ResourceId,
        material_data: &Self::DataType,
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

        Self {
            id,
            message_hub: message_hub.clone(),
            shared_data: shared_data.clone(),
            textures,
            material_index: INVALID_INDEX,
            path: PathBuf::new(),
        }
    }
}

impl Material {
    pub fn mark_as_dirty(&self) -> &Self {
        self.message_hub
            .send_event(ResourceEvent::<Self>::Changed(self.id));
        self
    }
    pub fn material_index(&self) -> i32 {
        self.material_index
    }
    pub fn set_material_index(&mut self, material_index: u32) -> bool {
        if self.material_index != material_index as i32 {
            self.material_index = material_index as _;
            return true;
        }
        false
    }

    pub fn textures(&self) -> &[Handle<Texture>; TextureType::Count as _] {
        &self.textures
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
}
