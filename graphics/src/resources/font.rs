use crate::{Font, MaterialInstance, MaterialRc, PipelineInstance, TextureInstance};
use nrg_filesystem::convert_from_local_path;
use nrg_math::Vector4;
use nrg_resources::{
    FileResource, ResourceData, ResourceId, ResourceRef, SharedData, SharedDataRw, DATA_FOLDER,
};
use nrg_serialize::{generate_uid_from_string, INVALID_UID};
use std::path::{Path, PathBuf};

pub type FontId = ResourceId;
pub type FontRc = ResourceRef<FontInstance>;
const UI_PIPELINE_NAME: &str = "UI";

pub struct FontInstance {
    id: ResourceId,
    path: PathBuf,
    material: MaterialRc,
    font: Font,
}

impl Default for FontInstance {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            path: PathBuf::new(),
            material: ResourceRef::default(),
            font: Font::default(),
        }
    }
}

impl ResourceData for FontInstance {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl FileResource for FontInstance {
    fn path(&self) -> &Path {
        self.path.as_path()
    }
    fn create_from_file(shared_data: &SharedDataRw, font_path: &Path) -> FontRc {
        if let Some(pipeline) = PipelineInstance::find_from_name(shared_data, UI_PIPELINE_NAME) {
            let path = convert_from_local_path(PathBuf::from(DATA_FOLDER).as_path(), font_path);
            if !path.exists() || !path.is_file() {
                panic!("Invalid font path {}", path.to_str().unwrap());
            }
            if let Some(font) = FontInstance::find_from_path(shared_data, path.as_path()) {
                return font;
            }
            let material = MaterialInstance::create_from_pipeline(shared_data, pipeline);
            let font = Font::new(path.as_path());
            let texture = if let Some(texture) =
                TextureInstance::find_from_path(shared_data, path.as_path())
            {
                texture
            } else {
                TextureInstance::create_from_file(shared_data, path.as_path())
            };
            material.resource().get_mut().add_texture(texture);

            SharedData::add_resource(
                shared_data,
                FontInstance {
                    id: generate_uid_from_string(path.to_str().unwrap()),
                    path,
                    material,
                    font,
                },
            )
        } else {
            panic!("No pipeline loaded with name {}", UI_PIPELINE_NAME);
        }
    }
}

impl FontInstance {
    pub fn find_from_path(shared_data: &SharedDataRw, font_path: &Path) -> Option<FontRc> {
        let path = convert_from_local_path(PathBuf::from(DATA_FOLDER).as_path(), font_path);
        SharedData::match_resource(shared_data, |f: &FontInstance| f.path == path)
    }
    pub fn get_default(shared_data: &SharedDataRw) -> FontId {
        if SharedData::has_resources_of_type::<FontInstance>(shared_data) {
            let fonts = SharedData::get_resources_of_type::<FontInstance>(shared_data);
            if !fonts.is_empty() {
                return fonts.first().unwrap().id();
            }
        }
        INVALID_UID
    }

    pub fn font(&self) -> &Font {
        &self.font
    }
    pub fn material(&self) -> MaterialRc {
        self.material.clone()
    }
    pub fn glyph_texture_coord(&self, c: char) -> Vector4 {
        let index = self.font.get_glyph_index(c);
        let texture_coord = self.font.get_glyph(index).texture_coord;
        texture_coord
    }
}
