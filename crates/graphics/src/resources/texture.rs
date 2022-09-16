use std::path::{Path, PathBuf};

use image::ImageFormat;
use inox_filesystem::{convert_from_local_path, File};
use inox_log::debug_log;
use inox_messenger::MessageHubRc;
use inox_resources::{
    Data, DataTypeResource, Handle, Resource, ResourceEvent, ResourceId, ResourceTrait,
    SerializableResource, SharedData, SharedDataRc,
};
use inox_serialize::inox_serializable::SerializableRegistryRc;
use inox_uid::generate_random_uid;

use crate::{TextureData, TextureFormat, TextureUsage, INVALID_INDEX};

pub type TextureId = ResourceId;

#[derive(Clone)]
pub struct Texture {
    id: TextureId,
    message_hub: MessageHubRc,
    shared_data: SharedDataRc,
    path: PathBuf,
    data: Option<Vec<u8>>,
    texture_index: i32,
    width: u32,
    height: u32,
    format: TextureFormat,
    usage: TextureUsage,
    update_from_gpu: bool,
}

impl ResourceTrait for Texture {
    type OnCreateData = ();

    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &TextureId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(
        &mut self,
        _shared_data: &SharedData,
        _message_hub: &MessageHubRc,
        _id: &TextureId,
    ) {
    }
    fn on_copy(&mut self, other: &Self)
    where
        Self: Sized,
    {
        *self = other.clone();
    }
}

impl DataTypeResource for Texture {
    type DataType = TextureData;
    type OnCreateData = <Self as ResourceTrait>::OnCreateData;

    fn new(id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            id,
            message_hub: message_hub.clone(),
            shared_data: shared_data.clone(),
            path: PathBuf::new(),
            data: None,
            texture_index: INVALID_INDEX,
            width: 0,
            height: 0,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsage::TextureBinding | TextureUsage::CopyDst,
            update_from_gpu: false,
        }
    }

    fn invalidate(&mut self) -> &mut Self {
        self.texture_index = INVALID_INDEX;
        debug_log!("Texture {:?} will be reloaded", self.path);
        self
    }
    fn is_initialized(&self) -> bool {
        self.texture_index != INVALID_INDEX
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: ResourceId,
        data: &Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let mut texture = Self::new(id, shared_data, message_hub);
        texture.width = data.width;
        texture.height = data.height;
        texture.format = data.format;
        texture.usage = data.usage;
        if let Some(image_data) = &data.data {
            texture.data = Some(image_data.clone());
        }
        texture
    }
}

impl SerializableResource for Texture {
    fn set_path(&mut self, path: &Path) -> &mut Self {
        self.path = path.to_path_buf();
        self
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn extension() -> &'static str {
        "png"
    }

    fn deserialize_data(
        path: &Path,
        _registry: &SerializableRegistryRc,
        mut f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
        let mut file = File::new(path);
        let filepath = path.to_path_buf();
        file.load(move |bytes| {
            let image_format = ImageFormat::from_path(filepath.as_path()).unwrap();
            let image_data =
                image::load_from_memory_with_format(bytes.as_slice(), image_format).unwrap();
            f(TextureData {
                width: image_data.width(),
                height: image_data.height(),
                format: TextureFormat::Rgba8Unorm,
                data: Some(image_data.into_rgba8().to_vec()),
                usage: TextureUsage::TextureBinding | TextureUsage::CopyDst,
            });
        });
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
    fn mark_as_dirty(&self) -> &Self {
        self.message_hub
            .send_event(ResourceEvent::<Self>::Changed(self.id));
        self
    }
    pub fn find_from_path(shared_data: &SharedDataRc, texture_path: &Path) -> Handle<Self> {
        let path = convert_from_local_path(Data::platform_data_folder().as_path(), texture_path);
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
    pub fn format(&self) -> TextureFormat {
        self.format
    }
    pub fn usage(&self) -> TextureUsage {
        self.usage
    }
    pub fn image_data(&self) -> &Option<Vec<u8>> {
        &self.data
    }
    pub fn texture_index(&self) -> i32 {
        self.texture_index
    }
    pub fn set_texture_index(&mut self, texture_index: usize) -> &mut Self {
        self.texture_index = texture_index as _;
        self
    }
    pub fn set_texture_size(&mut self, width: u32, height: u32) -> &mut Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn create_from_format(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        width: u32,
        height: u32,
        format: TextureFormat,
        usage: TextureUsage,
    ) -> Resource<Texture> {
        let texture_id = generate_random_uid();
        let mut texture = Texture::create_from_data(
            shared_data,
            message_hub,
            texture_id,
            &TextureData {
                width,
                height,
                format,
                data: None,
                usage,
            },
        );
        texture.on_create(shared_data, message_hub, &texture_id, None);
        shared_data.add_resource(message_hub, texture_id, texture)
    }

    fn image_data_from_format(width: u32, height: u32, format: TextureFormat) -> Vec<u8> {
        match format {
            crate::TextureFormat::R8Unorm
            | crate::TextureFormat::R8Uint
            | crate::TextureFormat::R8Snorm
            | crate::TextureFormat::R8Sint
            //| crate::TextureFormat::Stencil8 
            => {
                vec![0u8; ::std::mem::size_of::<u8>() * width as usize * height as usize]
            }
            crate::TextureFormat::R16Uint
            | crate::TextureFormat::R16Unorm
            | crate::TextureFormat::R16Float
            | crate::TextureFormat::R16Sint
            | crate::TextureFormat::R16Snorm
            /*| crate::TextureFormat::Depth16Unorm*/ => {
                vec![0u8; ::std::mem::size_of::<u16>() * width as usize * height as usize]
            }
            crate::TextureFormat::Rg8Unorm
            | crate::TextureFormat::Rg8Uint
            | crate::TextureFormat::Rg8Snorm
            | crate::TextureFormat::Rg8Sint => {
                let num_channels = 2;
                vec![
                    0u8;
                    num_channels * ::std::mem::size_of::<u8>() * width as usize * height as usize
                ]
            }
            crate::TextureFormat::R32Uint
            | crate::TextureFormat::R32Sint
            | crate::TextureFormat::R32Float
            | crate::TextureFormat::Rgb10a2Unorm
            | crate::TextureFormat::Rg11b10Float
            | crate::TextureFormat::Depth32Float
            | crate::TextureFormat::Depth24PlusStencil8
            | crate::TextureFormat::Rgb9e5Ufloat => {
                vec![0u8; ::std::mem::size_of::<u32>() * width as usize * height as usize]
            }
            crate::TextureFormat::Rgba8Unorm
            | crate::TextureFormat::Rgba8UnormSrgb
            | crate::TextureFormat::Rgba8Snorm
            | crate::TextureFormat::Rgba8Uint
            | crate::TextureFormat::Rgba8Sint
            | crate::TextureFormat::Bgra8Unorm
            | crate::TextureFormat::Bgra8UnormSrgb => {
                let num_channels = 4;
                vec![
                    0u8;
                    num_channels * ::std::mem::size_of::<u8>() * width as usize * height as usize
                ]
            }
            crate::TextureFormat::Rg16Uint
            | crate::TextureFormat::Rg16Sint
            | crate::TextureFormat::Rg16Unorm
            | crate::TextureFormat::Rg16Snorm
            | crate::TextureFormat::Rg16Float => {
                let num_channels = 2;
                vec![
                    0u8;
                    num_channels * ::std::mem::size_of::<u16>() * width as usize * height as usize
                ]
            }
            crate::TextureFormat::Rg32Uint
            | crate::TextureFormat::Rg32Sint
            | crate::TextureFormat::Rg32Float => {
                let num_channels = 2;
                vec![
                    0u8;
                    num_channels * ::std::mem::size_of::<u32>() * width as usize * height as usize
                ]
            }
            crate::TextureFormat::Rgba16Uint
            | crate::TextureFormat::Rgba16Sint
            | crate::TextureFormat::Rgba16Unorm
            | crate::TextureFormat::Rgba16Snorm
            | crate::TextureFormat::Rgba16Float => {
                let num_channels = 4;
                vec![
                    0u8;
                    num_channels * ::std::mem::size_of::<u16>() * width as usize * height as usize
                ]
            }
            crate::TextureFormat::Rgba32Uint
            | crate::TextureFormat::Rgba32Sint
            | crate::TextureFormat::Rgba32Float => {
                let num_channels = 4;
                vec![
                    0u8;
                    num_channels * ::std::mem::size_of::<u32>() * width as usize * height as usize
                ]
            }
            crate::TextureFormat::Depth32FloatStencil8 => {
                let num_channels = 5;
                vec![
                    0u8;
                    num_channels * ::std::mem::size_of::<u8>() * width as usize * height as usize
                ]
            }
            crate::TextureFormat::Depth24Plus => {
                let num_channels = 3;
                vec![
                    0u8;
                    num_channels * ::std::mem::size_of::<u8>() * width as usize * height as usize
                ]
            }
            _ => {
                panic!("Unsupported texture format: {:?}", format);
            }
        }
    }
}
