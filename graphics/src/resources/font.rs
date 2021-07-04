use crate::{Font, MaterialId, MaterialInstance, PipelineId, TextureInstance};
use nrg_math::Vector4;
use nrg_resources::{
    convert_from_local_path, ResourceId, ResourceTrait, SharedData, SharedDataRw, DATA_FOLDER,
};
use nrg_serialize::{generate_uid_from_string, INVALID_UID};
use std::path::{Path, PathBuf};

pub type FontId = ResourceId;

pub struct FontInstance {
    id: ResourceId,
    path: PathBuf,
    material_id: MaterialId,
    font: Font,
}

impl ResourceTrait for FontInstance {
    fn id(&self) -> ResourceId {
        self.id
    }
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl FontInstance {
    pub fn find_id(shared_data: &SharedDataRw, font_path: &Path) -> FontId {
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
    pub fn material(&self) -> MaterialId {
        self.material_id
    }

    pub fn get_material(shared_data: &SharedDataRw, font_id: FontId) -> MaterialId {
        let font = SharedData::get_resource::<FontInstance>(shared_data, font_id);
        let material_id = font.get().material();
        material_id
    }
    pub fn get_glyph_texture_coord(
        shared_data: &SharedDataRw,
        font_id: FontId,
        c: char,
    ) -> Vector4 {
        let font = SharedData::get_resource::<FontInstance>(shared_data, font_id);
        let index = font.get().font.get_glyph_index(c);
        let texture_coord = font.get().font.get_glyph(index).texture_coord;
        texture_coord
    }
    pub fn create_from_path(
        shared_data: &SharedDataRw,
        pipeline_id: PipelineId,
        font_path: &Path,
    ) -> FontId {
        let path = convert_from_local_path(PathBuf::from(DATA_FOLDER).as_path(), font_path);
        if !path.exists() || !path.is_file() || pipeline_id == INVALID_UID {
            eprintln!(
                "Invalid path {} or pipeline {}",
                path.to_str().unwrap(),
                pipeline_id.to_simple().to_string().as_str()
            );
            return INVALID_UID;
        }
        let font_id = FontInstance::find_id(shared_data, path.as_path());
        if font_id != INVALID_UID {
            return font_id;
        }
        let material_id = MaterialInstance::create_from_pipeline(shared_data, pipeline_id);
        let font = Font::new(path.as_path());
        let mut texture_id = TextureInstance::find_id(shared_data, path.as_path());
        if texture_id.is_nil() {
            texture_id = TextureInstance::create_from_file(shared_data, path.as_path());
        }
        MaterialInstance::add_texture(shared_data, material_id, texture_id);

        let mut data = shared_data.write().unwrap();
        data.add_resource(FontInstance {
            id: generate_uid_from_string(path.to_str().unwrap()),
            path,
            material_id,
            font,
        })
    }
}
