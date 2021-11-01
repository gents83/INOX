use crate::{FontData, Texture};

use nrg_math::Vector4;
use nrg_messenger::MessengerRw;
use nrg_resources::{
    DataTypeResource, Handle, ResourceId, SerializableResource, SharedData, SharedDataRc,
};
use nrg_serialize::{generate_random_uid, INVALID_UID};
use std::path::{Path, PathBuf};

pub type FontId = ResourceId;

#[derive(Default, Clone)]
pub struct Font {
    path: PathBuf,
    texture: Handle<Texture>,
    font_data: FontData,
}

impl DataTypeResource for Font {
    type DataType = FontData;

    fn create_from_data(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        _id: ResourceId,
        data: Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let mut font_data = data;
        let texture = Texture::new_resource(
            shared_data,
            global_messenger,
            generate_random_uid(),
            font_data.create_texture(),
        );
        Self {
            texture: Some(texture),
            font_data,
            ..Default::default()
        }
    }

    fn is_initialized(&self) -> bool {
        self.texture.is_some()
    }
    fn invalidate(&mut self) {
        self.texture = None;
    }
    fn deserialize_data(path: &Path) -> Self::DataType {
        FontData::new(path)
    }
}

impl SerializableResource for Font {
    fn set_path(&mut self, path: &Path) {
        self.path = path.to_path_buf();
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }
    fn is_matching_extension(path: &Path) -> bool {
        const FONT_EXTENSION: &str = "ttf";

        if let Some(ext) = path.extension().unwrap().to_str() {
            return ext == FONT_EXTENSION;
        }
        false
    }
}

impl Font {
    pub fn get_default(shared_data: &SharedDataRc) -> FontId {
        if let Some(font) = SharedData::match_resource(shared_data, |f: &Font| f.path().exists()) {
            return *font.id();
        }
        INVALID_UID
    }

    pub fn font_data(&self) -> &FontData {
        &self.font_data
    }
    pub fn texture(&self) -> &Handle<Texture> {
        &self.texture
    }
    pub fn glyph_texture_coord(&self, c: char) -> Vector4 {
        let texture_coord = self.font_data.get_glyph(c as _).texture_coord;
        texture_coord
    }
}
