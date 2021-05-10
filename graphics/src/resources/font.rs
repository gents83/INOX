use crate::{Font, MaterialId, MaterialInstance, PipelineId, TextureInstance};
use nrg_math::Vector4;
use nrg_resources::{ResourceId, ResourceTrait, SharedDataRw};
use nrg_serialize::INVALID_UID;
use std::path::{Path, PathBuf};

pub type FontId = ResourceId;

pub struct FontInstance {
    path: PathBuf,
    material_id: MaterialId,
    font: Font,
}

impl FontInstance {
    pub fn find_id(shared_data: &SharedDataRw, font_path: &Path) -> FontId {
        let data = shared_data.read().unwrap();
        data.match_resource(|f: &FontInstance| f.path == font_path)
    }
    pub fn get_default(shared_data: &SharedDataRw) -> FontId {
        let data = shared_data.read().unwrap();
        let fonts = data.get_resources_of_type::<FontInstance>();
        if !fonts.is_empty() {
            return fonts.first().unwrap().id();
        }
        INVALID_UID
    }

    pub fn get_material(shared_data: &SharedDataRw, font_id: FontId) -> MaterialId {
        let data = shared_data.read().unwrap();
        let font = data.get_resource::<FontInstance>(font_id);
        let material_id = font.get().material_id;
        material_id
    }
    pub fn get_glyph_texture_coord(
        shared_data: &SharedDataRw,
        font_id: FontId,
        c: char,
    ) -> Vector4 {
        let data = shared_data.read().unwrap();
        let font = data.get_resource::<FontInstance>(font_id);
        let index = font.get().font.get_glyph_index(c);
        let texture_coord = font.get().font.get_glyph(index).texture_coord;
        texture_coord
    }
    pub fn create_from_path(
        shared_data: &SharedDataRw,
        pipeline_id: PipelineId,
        font_path: &Path,
    ) -> FontId {
        if !font_path.exists() || pipeline_id == INVALID_UID {
            eprintln!(
                "Invalid path {} or pipeline {}",
                font_path.to_str().unwrap(),
                pipeline_id.to_simple().to_string().as_str()
            );
            return INVALID_UID;
        }
        let font_id = FontInstance::find_id(shared_data, font_path);
        if font_id != INVALID_UID {
            return font_id;
        }
        let material_id = MaterialInstance::create_from_pipeline(shared_data, pipeline_id);
        let font = Font::new(font_path);
        let texture_id = TextureInstance::find_id(shared_data, font.get_texture_path().as_path());
        if texture_id == INVALID_UID {
            let texture_id =
                TextureInstance::create_from_path(shared_data, font.get_texture_path().as_path());
            MaterialInstance::add_texture(shared_data, material_id, texture_id);
        }
        let mut data = shared_data.write().unwrap();
        data.add_resource(FontInstance {
            path: PathBuf::from(font_path),
            material_id,
            font,
        })
    }
}
