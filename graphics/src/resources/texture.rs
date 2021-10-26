use std::path::{Path, PathBuf};

use image::RgbaImage;
use nrg_filesystem::convert_from_local_path;
use nrg_messenger::MessengerRw;
use nrg_profiler::debug_log;
use nrg_resources::{
    DataTypeResource, Handle, Resource, ResourceId, SerializableResource, SharedData, SharedDataRc,
    DATA_FOLDER,
};

use crate::{
    api::backend::BackendPhysicalDevice, Device, TextureHandler, TextureInfo, INVALID_INDEX,
    TEXTURE_CHANNEL_COUNT,
};

pub type TextureId = ResourceId;

#[derive(Clone)]
pub struct Texture {
    path: PathBuf,
    image_data: Option<Vec<u8>>,
    texture_index: i32,
    layer_index: i32,
    width: u32,
    height: u32,
    is_initialized: bool,
    update_from_gpu: bool,
}

impl Default for Texture {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            image_data: None,
            texture_index: INVALID_INDEX,
            layer_index: INVALID_INDEX,
            width: 0,
            height: 0,
            is_initialized: false,
            update_from_gpu: false,
        }
    }
}

impl DataTypeResource for Texture {
    type DataType = RgbaImage;
    fn invalidate(&mut self) {
        self.is_initialized = false;
        debug_log(format!("Texture {:?} will be reloaded", self.path).as_str());
    }
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
    fn deserialize_data(path: &Path) -> Self::DataType {
        let image_data = image::open(path).unwrap();
        image_data.to_rgba8()
    }
    fn create_from_data(
        shared_data: &SharedDataRc,
        _global_messenger: &MessengerRw,
        id: TextureId,
        image_data: Self::DataType,
    ) -> Resource<Self> {
        let dimensions = image_data.dimensions();
        let texture = Self {
            image_data: Some(image_data.as_raw().clone()),
            width: dimensions.0,
            height: dimensions.1,
            ..Default::default()
        };
        SharedData::add_resource(shared_data, id, texture)
    }
}

impl SerializableResource for Texture {
    fn set_path(&mut self, path: &Path) {
        self.path = path.to_path_buf();
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn is_matching_extension(path: &Path) -> bool {
        const IMAGE_PNG_EXTENSION: &str = "png";
        const IMAGE_JPG_EXTENSION: &str = "jpg";
        const IMAGE_JPEG_EXTENSION: &str = "jpeg";
        const IMAGE_BMP_EXTENSION: &str = "bmp";
        const IMAGE_TGA_EXTENSION: &str = "tga";
        const IMAGE_DDS_EXTENSION: &str = "dds";
        const IMAGE_TIFF_EXTENSION: &str = "tiff";
        const IMAGE_GIF_EXTENSION: &str = "bmp";
        const IMAGE_ICO_EXTENSION: &str = "ico";

        if let Some(ext) = path.extension().unwrap().to_str() {
            return ext == IMAGE_PNG_EXTENSION
                || ext == IMAGE_JPG_EXTENSION
                || ext == IMAGE_JPEG_EXTENSION
                || ext == IMAGE_BMP_EXTENSION
                || ext == IMAGE_TGA_EXTENSION
                || ext == IMAGE_DDS_EXTENSION
                || ext == IMAGE_TIFF_EXTENSION
                || ext == IMAGE_GIF_EXTENSION
                || ext == IMAGE_ICO_EXTENSION;
        }
        false
    }
}

impl Texture {
    pub fn find_from_path(shared_data: &SharedDataRc, texture_path: &Path) -> Handle<Self> {
        let path = convert_from_local_path(PathBuf::from(DATA_FOLDER).as_path(), texture_path);
        SharedData::match_resource(shared_data, |t: &Texture| t.path == path)
    }
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn image_data(&self) -> &Option<Vec<u8>> {
        &self.image_data
    }
    pub fn set_texture_info(&mut self, texture_info: &TextureInfo) -> &mut Self {
        self.texture_index = texture_info.texture_index as _;
        self.layer_index = texture_info.layer_index as _;
        self.width = texture_info.area.width as u32;
        self.height = texture_info.area.height as u32;
        self.is_initialized = true;
        self
    }
    pub fn update_from_gpu(&self) -> bool {
        self.update_from_gpu
    }
    pub fn set_update_from_gpu(&mut self, should_update: bool) -> &mut Self {
        self.update_from_gpu = should_update;
        self
    }
    pub fn texture_index(&self) -> i32 {
        self.texture_index
    }
    pub fn layer_index(&self) -> i32 {
        self.layer_index
    }
    pub fn capture_image(
        &mut self,
        texture_id: &TextureId,
        texture_handler: &TextureHandler,
        device: &Device,
        physical_device: &BackendPhysicalDevice,
    ) {
        nrg_profiler::scoped_profile!("texture::capture_image");
        if self.image_data.is_none() {
            let mut image_data = Vec::new();
            image_data.resize_with(
                (self.width * self.height * TEXTURE_CHANNEL_COUNT) as _,
                || 0u8,
            );
            self.image_data = Some(image_data)
        }
        texture_handler.copy(
            device,
            physical_device,
            texture_id,
            self.image_data.as_mut().unwrap().as_mut_slice(),
        );
    }
}
