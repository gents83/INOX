use std::path::{Path, PathBuf};

use image::RgbaImage;
use sabi_filesystem::convert_from_local_path;
use sabi_messenger::MessageHubRc;
use sabi_profiler::debug_log;
use sabi_resources::{
    Data, DataTypeResource, Handle, ResourceId, ResourceTrait, SerializableResource, SharedData,
    SharedDataRc,
};

use crate::{RenderContext, TextureHandler, INVALID_INDEX, TEXTURE_CHANNEL_COUNT};

pub type TextureId = ResourceId;

#[derive(Clone)]
pub struct Texture {
    path: PathBuf,
    data: Option<Vec<u8>>,
    uniform_index: i32,
    width: u32,
    height: u32,
    update_from_gpu: bool,
}

impl Default for Texture {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            data: None,
            uniform_index: INVALID_INDEX,
            width: 0,
            height: 0,
            update_from_gpu: false,
        }
    }
}

impl DataTypeResource for Texture {
    type DataType = RgbaImage;
    type OnCreateData = ();

    fn invalidate(&mut self) -> &mut Self {
        self.uniform_index = INVALID_INDEX;
        debug_log(format!("Texture {:?} will be reloaded", self.path).as_str());
        self
    }
    fn is_initialized(&self) -> bool {
        self.uniform_index != INVALID_INDEX
    }
    fn deserialize_data(path: &Path) -> Self::DataType {
        let image_data = image::open(path).unwrap();
        image_data.to_rgba8()
    }
    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _messenger: &MessageHubRc,
        _id: &TextureId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(
        &mut self,
        _shared_data: &SharedData,
        _messenger: &MessageHubRc,
        _id: &TextureId,
    ) {
    }

    fn create_from_data(
        _shared_data: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: ResourceId,
        data: Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let dimensions = data.dimensions();
        Self {
            data: Some(data.as_raw().clone()),
            width: dimensions.0,
            height: dimensions.1,
            ..Default::default()
        }
    }
}

impl SerializableResource for Texture {
    fn set_path(&mut self, path: &Path) {
        self.path = path.to_path_buf();
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn extension() -> &'static str {
        "png"
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
        let path = convert_from_local_path(Data::data_folder().as_path(), texture_path);
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
        &self.data
    }
    pub fn uniform_index(&self) -> i32 {
        self.uniform_index
    }
    pub fn set_texture_data(&mut self, uniform_index: usize, width: u32, height: u32) -> &mut Self {
        self.uniform_index = uniform_index as _;
        self.width = width;
        self.height = height;
        self
    }
    pub fn update_from_gpu(&self) -> bool {
        self.update_from_gpu
    }
    pub fn set_update_from_gpu(&mut self, should_update: bool) -> &mut Self {
        self.update_from_gpu = should_update;
        self
    }
    pub fn capture_image(
        &mut self,
        texture_id: &TextureId,
        texture_handler: &TextureHandler,
        context: &RenderContext,
    ) {
        sabi_profiler::scoped_profile!("texture::capture_image");
        if self.data.is_none() {
            let mut image_data = Vec::new();
            image_data.resize_with(
                (self.width * self.height * TEXTURE_CHANNEL_COUNT) as _,
                || 0u8,
            );
            self.data = Some(image_data)
        }
        texture_handler.copy(
            context,
            texture_id,
            self.data.as_mut().unwrap().as_mut_slice(),
        );
    }
}
