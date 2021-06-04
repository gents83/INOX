use std::path::{Path, PathBuf};

use nrg_resources::{
    get_absolute_path_from, ResourceId, ResourceTrait, SharedData, SharedDataRw, DATA_FOLDER,
};
use nrg_serialize::{generate_uid_from_string, INVALID_UID};

use crate::INVALID_INDEX;

pub type TextureId = ResourceId;

pub struct TextureInstance {
    id: ResourceId,
    path: PathBuf,
    texture_index: i32,
    layer_index: i32,
    is_initialized: bool,
}

impl ResourceTrait for TextureInstance {
    fn id(&self) -> ResourceId {
        self.id
    }
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl TextureInstance {
    pub fn find_id(shared_data: &SharedDataRw, texture_path: &Path) -> TextureId {
        let path = get_absolute_path_from(PathBuf::from(DATA_FOLDER).as_path(), texture_path);
        SharedData::match_resource(shared_data, |t: &TextureInstance| t.path == path)
    }
    pub fn get_path(&self) -> &Path {
        self.path.as_path()
    }
    pub fn set_texture_data(&mut self, texture_index: u32, layer_index: u32) -> &mut Self {
        self.texture_index = texture_index as _;
        self.layer_index = layer_index as _;
        self.is_initialized = true;
        self
    }
    pub fn invalidate(&mut self) {
        self.is_initialized = false;
        println!("Texture {:?} will be reloaded", self.path);
    }
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }
    pub fn get_texture_index(&self) -> i32 {
        self.texture_index
    }
    pub fn get_layer_index(&self) -> i32 {
        self.layer_index
    }
    pub fn create(texture_path: &Path) -> TextureInstance {
        TextureInstance {
            id: generate_uid_from_string(texture_path.to_str().unwrap()),
            path: texture_path.to_path_buf(),
            texture_index: INVALID_INDEX,
            layer_index: INVALID_INDEX,
            is_initialized: false,
        }
    }

    pub fn create_from_path(shared_data: &SharedDataRw, texture_path: &Path) -> TextureId {
        let path = get_absolute_path_from(PathBuf::from(DATA_FOLDER).as_path(), texture_path);
        let texture_id =
            { SharedData::match_resource(shared_data, |t: &TextureInstance| t.path == path) };
        if texture_id != INVALID_UID {
            return texture_id;
        }
        let mut data = shared_data.write().unwrap();
        data.add_resource(TextureInstance::create(texture_path))
    }
}
