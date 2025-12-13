use crate::{
    FontData, Texture, TextureData, TextureFormat, TextureUsage, DEFAULT_FONT_TEXTURE_SIZE,
};

use inox_math::Vector4;
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, ResourceId, ResourceTrait, SerializableResource, SharedData,
    SharedDataRc,
};
use inox_uid::{generate_random_uid, INVALID_UID};
use std::path::{Path, PathBuf};

pub type FontId = ResourceId;

#[derive(Clone)]
pub struct Font {
    path: PathBuf,
    texture: Handle<Texture>,
    font_data: FontData,
}

impl ResourceTrait for Font {
    fn is_initialized(&self) -> bool {
        self.texture.is_some()
    }
    fn invalidate(&mut self) -> &mut Self {
        self.texture = None;
        self
    }
}

impl DataTypeResource for Font {
    type DataType = FontData;

    fn new(_id: ResourceId, _shared_data: &SharedDataRc, _message_hub: &MessageHubRc) -> Self {
        Self {
            path: PathBuf::new(),
            texture: None,
            font_data: FontData::default(),
        }
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        _id: ResourceId,
        data: &Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let mut font_data = data.clone();
        let texture_data = TextureData {
            width: DEFAULT_FONT_TEXTURE_SIZE as _,
            height: DEFAULT_FONT_TEXTURE_SIZE as _,
            data: Some(font_data.create_texture()),
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsage::TextureBinding | TextureUsage::CopyDst,
            sample_count: 1,
            layer_count: 1,
            is_LUT: false,
            mips_count: 1,
        };
        let texture = Texture::new_resource(
            shared_data,
            message_hub,
            generate_random_uid(),
            &texture_data,
            None,
        );
        Self {
            texture: Some(texture),
            font_data,
            path: PathBuf::new(),
        }
    }
}
impl SerializableResource for Font {
    fn set_path(&mut self, path: &Path) -> &mut Self {
        self.path = path.to_path_buf();
        self
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }
    fn extension() -> &'static str {
        "ttf"
    }
    fn deserialize_data(path: &Path, mut f: Box<dyn FnMut(Self::DataType) + 'static>) {
        f(FontData::new(path));
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
