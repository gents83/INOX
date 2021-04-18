use std::path::Path;

use image::{DynamicImage, EncodableLayout, Pixel};

use crate::{api::backend, Area, AreaAllocator, Pipeline, DEFAULT_AREA_SIZE, INVALID_INDEX};

use super::device::*;

pub const MAX_TEXTURE_COUNT: usize = 32;
pub const DEFAULT_LAYER_COUNT: usize = 32;

struct LayeredTexture {
    texture: backend::Texture,
    allocators: Vec<AreaAllocator>,
}

impl LayeredTexture {
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
    pub fn get_uv(&self) -> (f32, f32, f32, f32) {
        (
            self.area.x as f32 / DEFAULT_AREA_SIZE as f32,
            self.area.y as f32 / DEFAULT_AREA_SIZE as f32,
            (self.area.x as f32 + self.area.width as f32) / DEFAULT_AREA_SIZE as f32,
            (self.area.y as f32 + self.area.height as f32) / DEFAULT_AREA_SIZE as f32,
        )
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
    layered_textures: Vec<LayeredTexture>,
    textures: Vec<Texture>,
}

impl TextureHandler {
    pub fn create(device: &Device) -> Self {
        Self {
            device: device.clone(),
            layered_textures: vec![LayeredTexture::create(device)],
            textures: Vec::new(),
        }
    }

    pub fn get_texture(&self, index: usize) -> &Texture {
        &self.textures[index]
    }

    pub fn add(&mut self, filepath: &Path) -> usize {
        let image = image::open(filepath).unwrap();
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
            panic!("Unable to add texture {}", filepath.to_str().unwrap());
        }
        self.textures.push(Texture {
            texture_index: texture_index as _,
            layer_index: layer_index as _,
            area,
        });
        self.textures.len() - 1
    }

    pub fn add_empty(&mut self) -> usize {
        let image = DynamicImage::new_rgba8(16, 16);
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
            texture_index: texture_index as _,
            layer_index: layer_index as _,
            area,
        });
        self.textures.len() - 1
    }

    fn add_image(&mut self, width: u32, height: u32, image_data: &[u8]) -> (i32, i32, Area) {
        for (texture_index, layered_texture) in self.layered_textures.iter_mut().enumerate() {
            for (layer_index, area_allocator) in layered_texture.allocators.iter_mut().enumerate() {
                if let Some(area) = area_allocator.allocate(width, height) {
                    layered_texture.texture.add_in_layer(
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
    pub fn update_descriptor_sets(&self, pipeline: &Pipeline) {
        let mut textures: Vec<&backend::Texture> = Vec::new();
        for t in self.layered_textures.iter() {
            textures.push(t.get_texture());
        }
        pipeline.update_descriptor_sets(textures.as_slice());
    }
}
