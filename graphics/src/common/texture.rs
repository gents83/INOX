use std::path::Path;

use image::{DynamicImage, EncodableLayout, Pixel};
use nrg_serialize::{generate_random_uid, Uid};

use crate::{api::backend, Area, AreaAllocator, DEFAULT_AREA_SIZE, INVALID_INDEX};

use super::device::*;

pub const MAX_TEXTURE_COUNT: usize = 8;
pub const DEFAULT_LAYER_COUNT: usize = 8;

pub struct TextureAtlas {
    texture: backend::Texture,
    allocators: Vec<AreaAllocator>,
}

impl TextureAtlas {
    fn create(device: &Device) -> Self {
        let mut allocators: Vec<AreaAllocator> = Vec::new();
        for _i in 0..DEFAULT_LAYER_COUNT {
            allocators.push(AreaAllocator::default());
        }
        Self {
            texture: backend::Texture::create(
                &device.inner,
                DEFAULT_AREA_SIZE,
                DEFAULT_AREA_SIZE,
                DEFAULT_LAYER_COUNT,
            ),
            allocators,
        }
    }

    pub fn get_texture(&self) -> &backend::Texture {
        &self.texture
    }
}

pub struct Texture {
    id: Uid,
    texture_index: u32,
    layer_index: u32,
    area: Area,
}

impl Texture {
    pub fn get_texture_index(&self) -> u32 {
        self.texture_index
    }
    pub fn get_layer_index(&self) -> u32 {
        self.layer_index
    }
    pub fn get_height(&self) -> u32 {
        self.area.height
    }
    pub fn get_width(&self) -> u32 {
        self.area.width
    }
    pub fn convert_uv(&self, u: f32, v: f32) -> (f32, f32) {
        (
            (self.area.x as f32 + u * self.area.width as f32) / DEFAULT_AREA_SIZE as f32,
            (self.area.y as f32 + v * self.area.height as f32) / DEFAULT_AREA_SIZE as f32,
        )
    }
}

pub struct TextureHandler {
    device: Device,
    texture_atlas: Vec<TextureAtlas>,
    textures: Vec<Texture>,
}

impl TextureHandler {
    pub fn create(device: &Device) -> Self {
        let mut texture_handler = Self {
            device: device.clone(),
            texture_atlas: vec![TextureAtlas::create(device)],
            textures: Vec::new(),
        };
        texture_handler.add_empty();
        texture_handler
    }

    pub fn get_texture(&self, id: Uid) -> &Texture {
        let index = self.textures.iter().position(|t| t.id == id).unwrap();
        &self.textures[index]
    }

    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }

    pub fn add(&mut self, id: Uid, image: DynamicImage) -> (u32, u32) {
        let image_data = image.to_rgba8();
        let (texture_index, layer_index, area) = self.add_image(
            image_data.width(),
            image_data.height(),
            image_data.as_raw().as_bytes(),
        );
        if texture_index == INVALID_INDEX
            || layer_index < 0
            || layer_index >= DEFAULT_LAYER_COUNT as _
        {
            panic!(
                "Unable to add an image of size [{}x{}]",
                image_data.width(),
                image_data.height()
            );
        }
        self.textures.push(Texture {
            id,
            texture_index: texture_index as _,
            layer_index: layer_index as _,
            area,
        });
        (texture_index as _, layer_index as _)
    }

    pub fn add_from_path(&mut self, id: Uid, filepath: &Path) -> (u32, u32) {
        let image = image::open(filepath).unwrap();
        self.add(id, image)
    }

    pub fn remove(&mut self, id: Uid) {
        if let Some(index) = self.textures.iter().position(|t| t.id == id) {
            self.remove_image(
                self.textures[index].texture_index,
                self.textures[index].layer_index,
                self.textures[index].area.clone(),
            );
            self.textures.remove(index);
        } else {
            eprintln!(
                "Trying to remove a texture that doesn't exist with id {}",
                id.to_simple().to_string()
            );
        }
    }

    pub fn add_empty(&mut self) -> usize {
        let image = DynamicImage::new_rgba8(8, 8);
        let mut image_data = image.to_rgba8();
        let (width, height) = image_data.dimensions();
        for x in 0..width {
            for y in 0..height {
                image_data.put_pixel(x, y, Pixel::from_channels(255, 255, 255, 255))
            }
        }
        let (texture_index, layer_index, area) = self.add_image(
            image_data.width(),
            image_data.height(),
            image_data.as_raw().as_bytes(),
        );
        self.textures.push(Texture {
            id: generate_random_uid(),
            texture_index: texture_index as _,
            layer_index: layer_index as _,
            area,
        });
        self.textures.len() - 1
    }

    fn add_image(&mut self, width: u32, height: u32, image_data: &[u8]) -> (i32, i32, Area) {
        for (texture_index, texture_atlas) in self.texture_atlas.iter_mut().enumerate() {
            for (layer_index, area_allocator) in texture_atlas.allocators.iter_mut().enumerate() {
                if let Some(area) = area_allocator.allocate(width, height) {
                    texture_atlas.texture.add_in_layer(
                        &self.device.inner,
                        layer_index,
                        area,
                        image_data,
                    );
                    return (texture_index as _, layer_index as _, area.clone());
                }
            }
        }
        (INVALID_INDEX, INVALID_INDEX, Area::default())
    }

    fn remove_image(&mut self, texture_index: u32, layer_index: u32, area: Area) {
        let texture_atlas = &mut self.texture_atlas[texture_index as usize];
        let allocator = &mut texture_atlas.allocators[layer_index as usize];
        allocator.remove(area);
    }

    pub fn get_textures(&self) -> &[TextureAtlas] {
        self.texture_atlas.as_slice()
    }
}

pub fn is_texture(path: &Path) -> bool {
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
