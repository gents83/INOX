use std::path::{Path, PathBuf};

use nrg_resources::{ResourceId, SharedData, SharedDataRw};
use nrg_serialize::INVALID_UID;

use crate::INVALID_INDEX;

pub type TextureId = ResourceId;

pub struct TextureInstance {
    path: PathBuf,
    texture_handler_index: i32,
    texture_index: i32,
    layer_index: i32,
}

impl Default for TextureInstance {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            texture_handler_index: INVALID_INDEX,
            texture_index: INVALID_INDEX,
            layer_index: INVALID_INDEX,
        }
    }
}

impl TextureInstance {
    pub fn find_id(shared_data: &SharedDataRw, texture_path: &Path) -> TextureId {
        SharedData::match_resource(shared_data, |t: &TextureInstance| t.path == texture_path)
    }
    pub fn get_path(&self) -> &Path {
        self.path.as_path()
    }
    pub fn set_texture_data(
        &mut self,
        texture_handler_index: u32,
        texture_index: u32,
        layer_index: u32,
    ) -> &mut Self {
        self.texture_handler_index = texture_handler_index as _;
        self.texture_index = texture_index as _;
        self.layer_index = layer_index as _;
        self
    }
    pub fn get_texture_handler_index(&self) -> i32 {
        self.texture_handler_index
    }
    pub fn get_texture_index(&self) -> i32 {
        self.texture_index
    }
    pub fn get_layer_index(&self) -> i32 {
        self.layer_index
    }
    pub fn create_from_path(shared_data: &SharedDataRw, texture_path: &Path) -> TextureId {
        let texture_id = {
            SharedData::match_resource(shared_data, |t: &TextureInstance| t.path == texture_path)
        };
        if texture_id != INVALID_UID {
            return texture_id;
        }
        let mut data = shared_data.write().unwrap();
        data.add_resource(TextureInstance {
            path: PathBuf::from(texture_path),
            texture_handler_index: INVALID_INDEX,
            texture_index: INVALID_INDEX,
            layer_index: INVALID_INDEX,
        })
    }
}
