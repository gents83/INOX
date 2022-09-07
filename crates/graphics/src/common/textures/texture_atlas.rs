use inox_log::debug_log;
use inox_uid::generate_random_uid;

use crate::{TextureFormat, TextureId, TextureInfo};

use super::{
    area::{Area, AreaAllocator, DEFAULT_AREA_SIZE},
    texture::Texture,
};

pub const DEFAULT_LAYER_COUNT: u32 = 8u32;
pub const MAX_TEXTURE_ATLAS_COUNT: u32 = 15u32;

pub struct TextureAtlas {
    texture: Texture,
    allocators: Vec<AreaAllocator>,
}

impl TextureAtlas {
    pub fn create_default(device: &wgpu::Device, format: TextureFormat) -> Self {
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
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        Self {
            texture,
            allocators,
        }
    }

    pub fn destroy(&mut self) {
        self.texture.release();
    }

    pub fn texture_id(&self) -> &TextureId {
        self.texture.id()
    }
    pub fn texture_view(&self) -> &wgpu::TextureView {
        self.texture.view()
    }
    pub fn texture_format(&self) -> &TextureFormat {
        self.texture.format()
    }
    pub fn width(&self) -> u32 {
        self.texture.width()
    }
    pub fn height(&self) -> u32 {
        self.texture.height()
    }

    pub fn get_area(&self, texture_id: &TextureId) -> Option<&Area> {
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
    ) -> Option<TextureInfo> {
        for (layer_index, area_allocator) in self.allocators.iter_mut().enumerate() {
            if let Some(area) = area_allocator.allocate(id, dimensions.0, dimensions.1) {
                self.texture
                    .send_to_gpu(device, encoder, layer_index as _, area, image_data);
                return Some(TextureInfo {
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

    pub fn texture_info(&self, texture_index: u32, texture_id: &TextureId) -> Option<TextureInfo> {
        for (layer_index, area_allocator) in self.allocators.iter().enumerate() {
            if let Some(area) = area_allocator.get_area(texture_id) {
                return Some(TextureInfo {
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

    pub fn remove(&mut self, texture_id: &TextureId) -> bool {
        for (layer_index, allocator) in self.allocators.iter_mut().enumerate() {
            if allocator.remove_texture(texture_id) {
                //todo remove the real texture from device memory
                //atlas.texture.remove_from_layer(device, layer_index, &area);
                debug_log!(
                    "Removing from texture atlas with format {:?} at layer {:}",
                    self.texture.format(),
                    layer_index
                )
            }
        }
        self.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.allocators.iter().all(|a| a.is_empty())
    }
}
