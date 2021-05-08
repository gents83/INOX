use std::path::{Path, PathBuf};

use nrg_resources::{ResourceId, SharedDataRw};
use nrg_serialize::INVALID_UID;

use crate::INVALID_INDEX;

pub type TextureId = ResourceId;

pub struct TextureInstance {
    path: PathBuf,
    texture_index: i32,
}

impl Default for TextureInstance {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            texture_index: 0,
        }
    }
}

impl TextureInstance {
    pub fn find_id(shared_data: &SharedDataRw, texture_path: &Path) -> TextureId {
        let data = shared_data.read().unwrap();
        data.match_resource(|t: &TextureInstance| t.path == texture_path)
    }
    pub fn get_path(&self) -> &Path {
        self.path.as_path()
    }
    pub fn set_texture_index(&mut self, texture_index: u32) -> &mut Self {
        self.texture_index = texture_index as _;
        self
    }
    pub fn get_texture_index(&self) -> i32 {
        self.texture_index
    }
    pub fn create_from_path(shared_data: &SharedDataRw, texture_path: &Path) -> TextureId {
        let mut data = shared_data.write().unwrap();
        let texture_id = data.match_resource(|t: &TextureInstance| t.path == texture_path);
        if texture_id != INVALID_UID {
            return texture_id;
        }
        data.add_resource(TextureInstance {
            path: PathBuf::from(texture_path),
            texture_index: INVALID_INDEX,
        })
    }
}
