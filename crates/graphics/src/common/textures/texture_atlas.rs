use inox_log::debug_log;
use inox_uid::generate_random_uid;

use crate::{TextureData, TextureId};

use super::{
    area::{Area, AreaAllocator, DEFAULT_AREA_SIZE},
    texture::Texture,
};

pub const DEFAULT_LAYER_COUNT: u32 = 8u32;
pub const MAX_TEXTURE_ATLAS_COUNT: u32 = 16u32;

pub struct TextureAtlas {
    texture: Texture,
    allocators: Vec<AreaAllocator>,
}

impl TextureAtlas {
    pub fn create_default(device: &wgpu::Device) -> Self {
        let mut allocators: Vec<AreaAllocator> = Vec::new();
        for _i in 0..DEFAULT_LAYER_COUNT {
            allocators.push(AreaAllocator::new(DEFAULT_AREA_SIZE, DEFAULT_AREA_SIZE));
        }
        let texture = Texture::create(
            device,
            generate_random_uid(),
            DEFAULT_AREA_SIZE,
            DEFAULT_AREA_SIZE,
            DEFAULT_LAYER_COUNT,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        Self {
            texture,
            allocators,
        }
    }

    pub fn create_texture(
        device: &wgpu::Device,
        id: &TextureId,
        width: u32,
        height: u32,
        layers_count: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let mut area_allocator = AreaAllocator::new(width, height);
        if area_allocator.allocate(id, width, height).is_none() {
            panic!("Unable to create render target");
        }
        let texture = Texture::create(device, *id, width, height, layers_count, format, usage);
        Self {
            texture,
            allocators: vec![area_allocator],
        }
    }

    pub fn destroy(&mut self) {
        self.texture.release();
    }

    pub fn texture_id(&self) -> &TextureId {
        self.texture.id()
    }
    pub fn texture(&self) -> &wgpu::TextureView {
        self.texture.view()
    }
    pub fn texture_format(&self) -> &wgpu::TextureFormat {
        self.texture.format()
    }
    pub fn width(&self) -> u32 {
        self.texture.width()
    }
    pub fn height(&self) -> u32 {
        self.texture.height()
    }

    pub fn get_area(&self, texture_id: &TextureId) -> Option<Area> {
        for allocator in &self.allocators {
            if let Some(area) = allocator.get_area(texture_id) {
                return Some(area);
            }
        }
        None
    }

    pub fn allocate(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        id: &TextureId,
        texture_index: u32,
        dimensions: (u32, u32),
        image_data: &[u8],
    ) -> Option<TextureData> {
        for (layer_index, area_allocator) in self.allocators.iter_mut().enumerate() {
            if let Some(area) = area_allocator.allocate(id, dimensions.0, dimensions.1) {
                self.texture
                    .send_to_gpu(device, encoder, layer_index as _, area, image_data);
                return Some(TextureData {
                    texture_index: texture_index as _,
                    layer_index: layer_index as _,
                    area: area.into(),
                    total_width: self.texture.width() as _,
                    total_height: self.texture.height() as _,
                });
            }
        }
        None
    }

    pub fn get_texture_data(
        &self,
        texture_index: u32,
        texture_id: &TextureId,
    ) -> Option<TextureData> {
        for (layer_index, area_allocator) in self.allocators.iter().enumerate() {
            if let Some(area) = area_allocator.get_area(texture_id) {
                return Some(TextureData {
                    texture_index: texture_index as _,
                    layer_index: layer_index as _,
                    area: area.into(),
                    total_width: self.texture.width() as _,
                    total_height: self.texture.height() as _,
                });
            }
        }
        None
    }

    pub fn remove(&mut self, atlas_index: u32, texture_id: &TextureId) -> bool {
        for (layer_index, allocator) in self.allocators.iter_mut().enumerate() {
            if allocator.remove_texture(texture_id) {
                //todo remove the real texture from device memory
                //atlas.texture.remove_from_layer(device, layer_index, &area);
                debug_log!(
                    "Removing from texture atlas {:?} at layer {:}",
                    atlas_index,
                    layer_index
                );
                if self.allocators.iter().all(|a| a.is_empty()) {
                    return true;
                }
                return false;
            }
        }
        false
    }

    pub fn read_from_gpu(&self, device: &wgpu::Device, texture_id: &TextureId) -> bool {
        for (layer_index, allocator) in self.allocators.iter().enumerate() {
            if let Some(area) = allocator.get_area(texture_id) {
                self.texture.read_from_gpu(device, &area, layer_index as _);
                return true;
            }
        }
        false
    }
}
