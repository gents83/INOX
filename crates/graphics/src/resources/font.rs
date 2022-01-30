use crate::{FontData, Texture};

use sabi_math::Vector4;
use sabi_messenger::MessengerRw;
use sabi_resources::{
    DataTypeResource, Handle, ResourceId, ResourceTrait, SerializableResource, SharedData,
    SharedDataRc,
};
use sabi_serialize::{generate_random_uid, INVALID_UID};
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
    type OnCreateData = ();

    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _id: &FontId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(&mut self, _shared_data: &SharedData, _messenger: &MessengerRw, _id: &FontId) {}

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
    fn invalidate(&mut self) -> &mut Self {
        self.texture = None;
        self
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
    fn extension() -> &'static str {
        "ttf"
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
