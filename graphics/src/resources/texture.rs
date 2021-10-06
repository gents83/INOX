use std::path::{Path, PathBuf};

use image::RgbaImage;
use nrg_filesystem::convert_from_local_path;
use nrg_resources::{
    DataTypeResource, FileResource, Handle, Resource, ResourceData, ResourceId, SharedData,
    SharedDataRw, DATA_FOLDER,
};
use nrg_serialize::{generate_random_uid, generate_uid_from_string, INVALID_UID};

use crate::{
    api::backend::BackendPhysicalDevice, Device, TextureHandler, TextureInfo, INVALID_INDEX,
    TEXTURE_CHANNEL_COUNT,
};

pub type TextureId = ResourceId;

pub struct Texture {
    id: ResourceId,
    path: PathBuf,
    image_data: Option<RgbaImage>,
    texture_index: i32,
    layer_index: i32,
    width: u32,
    height: u32,
    is_initialized: bool,
    is_render_target: bool,
}

impl Default for Texture {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            path: PathBuf::new(),
            image_data: None,
            texture_index: INVALID_INDEX,
            layer_index: INVALID_INDEX,
            width: 0,
            height: 0,
            is_initialized: false,
            is_render_target: false,
        }
    }
}

impl ResourceData for Texture {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl DataTypeResource for Texture {
    type DataType = RgbaImage;
    fn create_from_data(shared_data: &SharedDataRw, image_data: Self::DataType) -> Resource<Self> {
        let dimensions = image_data.dimensions();
        SharedData::add_resource(
            shared_data,
            Texture {
                id: generate_random_uid(),
                image_data: Some(image_data),
                width: dimensions.0,
                height: dimensions.1,
                ..Default::default()
            },
        )
    }
}

impl FileResource for Texture {
    fn path(&self) -> &Path {
        self.path.as_path()
    }
    fn create_from_file(shared_data: &SharedDataRw, filepath: &Path) -> Resource<Self> {
        let path = convert_from_local_path(PathBuf::from(DATA_FOLDER).as_path(), filepath);
        if let Some(texture) = SharedData::match_resource(shared_data, |t: &Texture| t.path == path)
        {
            return texture;
        }
        SharedData::add_resource(shared_data, Texture::create(filepath))
    }
}

impl Texture {
    pub fn find_from_path(shared_data: &SharedDataRw, texture_path: &Path) -> Handle<Self> {
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
    pub fn image_data(&mut self) -> Option<RgbaImage> {
        self.image_data.take()
    }
    pub fn set_texture_info(&mut self, texture_info: &TextureInfo) -> &mut Self {
        self.texture_index = texture_info.texture_index as _;
        self.layer_index = texture_info.layer_index as _;
        self.width = texture_info.area.width as u32;
        self.height = texture_info.area.height as u32;
        self.is_initialized = true;
        self
    }
    pub fn invalidate(&mut self) {
        self.is_initialized = false;
        println!("Texture {:?} will be reloaded", self.path);
    }
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }
    pub fn is_render_target(&self) -> bool {
        self.is_render_target
    }
    pub fn mark_as_render_taget(&mut self) -> &mut Self {
        self.is_render_target = true;
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
        texture_handler: &TextureHandler,
        device: &Device,
        physical_device: &BackendPhysicalDevice,
    ) {
        let mut image_data = Vec::new();
        image_data.resize_with(
            (self.width * self.height * TEXTURE_CHANNEL_COUNT) as _,
            || 0u8,
        );
        texture_handler.copy(device, physical_device, self.id, image_data.as_mut_slice());
        self.image_data = RgbaImage::from_raw(self.width, self.height, image_data);
    }
    fn create(texture_path: &Path) -> Texture {
        Texture {
            id: generate_uid_from_string(texture_path.to_str().unwrap()),
            path: texture_path.to_path_buf(),
            ..Default::default()
        }
    }
}
