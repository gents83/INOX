use crate::{FontData, Texture};
use nrg_filesystem::convert_from_local_path;
use nrg_math::Vector4;
use nrg_resources::{
    FileResource, Handle, Resource, ResourceData, ResourceId, SharedData, SharedDataRw, DATA_FOLDER,
};
use nrg_serialize::{generate_uid_from_string, INVALID_UID};
use std::path::{Path, PathBuf};

pub type FontId = ResourceId;

#[derive(Default)]
pub struct Font {
    id: ResourceId,
    path: PathBuf,
    texture: Handle<Texture>,
    font_data: FontData,
}

impl ResourceData for Font {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl FileResource for Font {
    fn path(&self) -> &Path {
        self.path.as_path()
    }
    fn create_from_file(shared_data: &SharedDataRw, font_path: &Path) -> Resource<Font> {
        let path = convert_from_local_path(PathBuf::from(DATA_FOLDER).as_path(), font_path);
        if !path.exists() || !path.is_file() {
            panic!("Invalid font path {}", path.to_str().unwrap());
        }
        if let Some(font) = Font::find_from_path(shared_data, path.as_path()) {
            return font;
        }
        let font = FontData::new(path.as_path());
        let texture = if let Some(texture) = Texture::find_from_path(shared_data, path.as_path()) {
            texture
        } else {
            Texture::create_from_file(shared_data, path.as_path())
        };

        SharedData::add_resource(
            shared_data,
            Font {
                id: generate_uid_from_string(path.to_str().unwrap()),
                path,
                texture: Some(texture),
                font_data: font,
            },
        )
    }
}

impl Font {
    pub fn find_from_path(shared_data: &SharedDataRw, font_path: &Path) -> Handle<Font> {
        let path = convert_from_local_path(PathBuf::from(DATA_FOLDER).as_path(), font_path);
        SharedData::match_resource(shared_data, |f: &Font| f.path == path)
    }
    pub fn get_default(shared_data: &SharedDataRw) -> FontId {
        if let Some(font) = SharedData::match_resource(shared_data, |f: &Font| !f.id().is_nil()) {
            return font.id();
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
        let index = self.font_data.get_glyph_index(c);
        let texture_coord = self.font_data.get_glyph(index).texture_coord;
        texture_coord
    }
}
