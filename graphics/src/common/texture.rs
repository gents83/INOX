use std::path::Path;

use image::{EncodableLayout, RgbaImage};
use nrg_serialize::{generate_random_uid, Uid};

use crate::{api::backend, Area, AreaAllocator, DEFAULT_AREA_SIZE};

use super::device::*;

pub const MAX_TEXTURE_COUNT: usize = 8;
pub const DEFAULT_LAYER_COUNT: usize = 8;

pub struct TextureAtlas {
    id: Uid,
    texture: backend::Texture,
    allocators: Vec<AreaAllocator>,
    info: Vec<TextureInfo>,
}

impl TextureAtlas {
    fn create(device: &Device) -> Self {
        let mut allocators: Vec<AreaAllocator> = Vec::new();
        for _i in 0..DEFAULT_LAYER_COUNT {
            allocators.push(AreaAllocator::new(DEFAULT_AREA_SIZE, DEFAULT_AREA_SIZE));
        }
        Self {
            id: generate_random_uid(),
            texture: backend::Texture::create(
                &device.inner,
                DEFAULT_AREA_SIZE,
                DEFAULT_AREA_SIZE,
                DEFAULT_LAYER_COUNT,
            ),
            allocators,
            info: Vec::new(),
        }
    }

    fn create_as_render_target(
        device: &Device,
        id: Uid,
        index: u32,
        width: u32,
        height: u32,
        is_depth: bool,
    ) -> Self {
        let mut area_allocator = AreaAllocator::new(width, height);
        if area_allocator.allocate(width, height).is_none() {
            panic!("Unable to create render target");
        }
        Self {
            id,
            texture: backend::Texture::create_as_render_target(
                &device.inner,
                width,
                height,
                1,
                is_depth,
            ),
            allocators: vec![area_allocator],
            info: vec![TextureInfo {
                id,
                texture_index: index,
                layer_index: 0,
                area: Area {
                    x: 0,
                    y: 0,
                    width,
                    height,
                },
            }],
        }
    }

    pub fn get_texture(&self) -> &backend::Texture {
        &self.texture
    }

    pub fn get_texture_info(&self, id: Uid) -> Option<&TextureInfo> {
        if let Some(index) = self.info.iter().position(|info| info.id == id) {
            return Some(&self.info[index]);
        }
        None
    }

    pub fn remove(&mut self, id: Uid) -> Option<TextureInfo> {
        if let Some(index) = self.info.iter().position(|t| t.id == id) {
            return Some(self.info.remove(index));
        }
        eprintln!(
            "Trying to remove a texture that doesn't exist with id {}",
            id.to_simple().to_string()
        );
        None
    }
}

#[derive(Clone, Copy)]
pub struct TextureInfo {
    pub id: Uid,
    pub texture_index: u32,
    pub layer_index: u32,
    pub area: Area,
}

impl TextureInfo {
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
            (self.area.x as f32 + 0.5 + u * self.area.width as f32) / DEFAULT_AREA_SIZE as f32,
            (self.area.y as f32 + 0.5 + v * self.area.height as f32) / DEFAULT_AREA_SIZE as f32,
        )
    }
}

pub struct TextureHandler {
    device: Device,
    texture_atlas: Vec<TextureAtlas>,
}

impl TextureHandler {
    pub fn create(device: &Device) -> Self {
        Self {
            device: device.clone(),
            texture_atlas: vec![TextureAtlas::create(device)],
        }
    }

    pub fn add_render_target(
        &mut self,
        device: &Device,
        id: Uid,
        width: u32,
        height: u32,
        is_depth: bool,
    ) {
        let index = self.texture_atlas.len() as _;
        self.texture_atlas
            .push(TextureAtlas::create_as_render_target(
                device, id, index, width, height, is_depth,
            ));
    }

    pub fn get_textures_atlas(&self) -> &[TextureAtlas] {
        self.texture_atlas.as_slice()
    }

    pub fn get_texture_atlas(&self, id: Uid) -> Option<&TextureAtlas> {
        if let Some(index) = self.texture_atlas.iter().position(|t| t.id == id) {
            return Some(&self.texture_atlas[index]);
        }
        None
    }

    pub fn get_texture_info(&self, id: Uid) -> Option<&TextureInfo> {
        if let Some(texture_atlas) = self.texture_atlas.iter().find(|t| {
            let texture_info = t.get_texture_info(id);
            texture_info.is_some()
        }) {
            return texture_atlas.get_texture_info(id);
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.texture_atlas.is_empty()
    }

    pub fn add(&mut self, id: Uid, image_data: RgbaImage) -> TextureInfo {
        self.add_image(
            id,
            image_data.width(),
            image_data.height(),
            image_data.as_raw().as_bytes(),
        )
    }

    pub fn remove(&mut self, id: Uid) {
        let mut texture_infos = Vec::new();
        self.texture_atlas.iter_mut().for_each(|t| {
            if let Some(texture_info) = t.remove(id) {
                texture_infos.push(texture_info);
            }
        });
        for info in texture_infos {
            self.remove_image(info.texture_index, info.layer_index, info.area);
        }
    }

    pub fn add_from_path(&mut self, id: Uid, filepath: &Path) -> TextureInfo {
        let image = image::open(filepath).unwrap();
        self.add(id, image.to_rgba8())
    }

    fn add_image(&mut self, id: Uid, width: u32, height: u32, image_data: &[u8]) -> TextureInfo {
        for (texture_index, texture_atlas) in self.texture_atlas.iter_mut().enumerate() {
            for (layer_index, area_allocator) in texture_atlas.allocators.iter_mut().enumerate() {
                if let Some(area) = area_allocator.allocate(width, height) {
                    texture_atlas.texture.add_in_layer(
                        &self.device.inner,
                        layer_index,
                        area,
                        image_data,
                    );
                    let info = TextureInfo {
                        id,
                        texture_index: texture_index as _,
                        layer_index: layer_index as _,
                        area: *area,
                    };
                    texture_atlas.info.push(info);
                    return info;
                }
            }
        }
        panic!("Unable to allocate texture")
    }

    fn remove_image(&mut self, texture_index: u32, layer_index: u32, area: Area) {
        let texture_atlas = &mut self.texture_atlas[texture_index as usize];
        let allocator = &mut texture_atlas.allocators[layer_index as usize];
        allocator.remove(area);
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
