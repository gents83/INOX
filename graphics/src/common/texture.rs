use std::path::Path;

use nrg_profiler::debug_log;
use nrg_serialize::generate_random_uid;

use crate::{
    api::backend::{self, BackendPhysicalDevice},
    Area, AreaAllocator, ShaderTextureData, TextureId, DEFAULT_AREA_SIZE,
};

use super::device::*;

pub const MAX_TEXTURE_ATLAS_COUNT: usize = 32;
pub const DEFAULT_LAYER_COUNT: u32 = 8;
pub const TEXTURE_CHANNEL_COUNT: u32 = 4;

pub struct TextureAtlas {
    id: TextureId,
    texture: backend::BackendTexture,
    allocators: Vec<AreaAllocator>,
}

impl TextureAtlas {
    fn create(device: &Device, physical_device: &BackendPhysicalDevice) -> Self {
        let mut allocators: Vec<AreaAllocator> = Vec::new();
        for _i in 0..DEFAULT_LAYER_COUNT {
            allocators.push(AreaAllocator::new(DEFAULT_AREA_SIZE, DEFAULT_AREA_SIZE));
        }
        Self {
            id: generate_random_uid(),
            texture: backend::BackendTexture::create(
                device,
                physical_device,
                DEFAULT_AREA_SIZE,
                DEFAULT_AREA_SIZE,
                DEFAULT_LAYER_COUNT,
            ),
            allocators,
        }
    }

    fn create_as_render_target(
        device: &Device,
        physical_device: &BackendPhysicalDevice,
        id: &TextureId,
        width: u32,
        height: u32,
        is_depth: bool,
    ) -> Self {
        let mut area_allocator = AreaAllocator::new(width, height);
        if area_allocator.allocate(id, width, height).is_none() {
            panic!("Unable to create render target");
        }
        Self {
            id: *id,
            texture: backend::BackendTexture::create_as_render_target(
                device,
                physical_device,
                width,
                height,
                1,
                is_depth,
            ),
            allocators: vec![area_allocator],
        }
    }

    pub fn destroy(&self, device: &Device) {
        self.texture.destroy(device);
    }

    pub fn get_texture(&self) -> &backend::BackendTexture {
        &self.texture
    }

    pub fn get_area(&self, texture_id: &TextureId) -> Option<Area> {
        for allocator in &self.allocators {
            if let Some(area) = allocator.get_area(texture_id) {
                return Some(area);
            }
        }
        None
    }
}

pub struct TextureHandler {
    texture_atlas: Vec<TextureAtlas>,
}

impl TextureHandler {
    pub fn create(device: &Device, physical_device: &BackendPhysicalDevice) -> Self {
        Self {
            texture_atlas: vec![TextureAtlas::create(device, physical_device)],
        }
    }

    pub fn add_render_target(
        &mut self,
        device: &Device,
        physical_device: &BackendPhysicalDevice,
        id: &TextureId,
        width: u32,
        height: u32,
        is_depth: bool,
    ) {
        self.texture_atlas
            .push(TextureAtlas::create_as_render_target(
                device,
                physical_device,
                id,
                width,
                height,
                is_depth,
            ));
    }

    pub fn get_textures_atlas(&self) -> &[TextureAtlas] {
        self.texture_atlas.as_slice()
    }

    pub fn get_texture_atlas(&self, id: &TextureId) -> Option<&TextureAtlas> {
        if let Some(index) = self.texture_atlas.iter().position(|t| t.id == *id) {
            return Some(&self.texture_atlas[index]);
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.texture_atlas.is_empty()
    }

    pub fn copy(
        &self,
        device: &Device,
        physical_device: &BackendPhysicalDevice,
        id: &TextureId,
        image_data: &mut [u8],
    ) {
        nrg_profiler::scoped_profile!("texture::copy");

        self.texture_atlas.iter().for_each(|atlas| {
            for (layer_index, allocator) in atlas.allocators.iter().enumerate() {
                if let Some(area) = allocator.get_area(id) {
                    atlas.texture.get_from_layer(
                        device,
                        physical_device,
                        layer_index as _,
                        &area,
                        image_data,
                    );
                    return;
                }
            }
        });
    }

    pub fn remove(&mut self, device: &Device, id: &TextureId) {
        let mut texture_to_remove = Vec::new();
        self.texture_atlas
            .iter_mut()
            .enumerate()
            .for_each(|(texture_index, atlas)| {
                for (layer_index, allocator) in atlas.allocators.iter_mut().enumerate() {
                    if allocator.remove_texture(id) {
                        //todo remove the real texture from device memory
                        //atlas.texture.remove_from_layer(device, layer_index, &area);
                        debug_log(
                            format!(
                                "Removing from texture atlas {:?} at layer {:}",
                                texture_index, layer_index
                            )
                            .as_str(),
                        );
                        if atlas.allocators.iter().all(|a| a.is_empty()) {
                            texture_to_remove.push(texture_index);
                        }
                        return;
                    }
                }
            });
        texture_to_remove.iter().rev().for_each(|i| {
            let texture_atlas = self.texture_atlas.remove(*i);
            texture_atlas.destroy(device);
            debug_log(format!("Removing texture atlas {:?}", i).as_str());
        });
    }

    pub fn add_from_path(
        &mut self,
        device: &Device,
        physical_device: &BackendPhysicalDevice,
        id: &TextureId,
        filepath: &Path,
    ) -> ShaderTextureData {
        let image = image::open(filepath).unwrap();
        self.add_image(
            device,
            physical_device,
            id,
            image.width(),
            image.height(),
            image.to_rgba8().as_raw().as_slice(),
        )
    }

    pub fn add_image(
        &mut self,
        device: &Device,
        physical_device: &BackendPhysicalDevice,
        id: &TextureId,
        width: u32,
        height: u32,
        image_data: &[u8],
    ) -> ShaderTextureData {
        for (texture_index, texture_atlas) in self.texture_atlas.iter_mut().enumerate() {
            for (layer_index, area_allocator) in texture_atlas.allocators.iter_mut().enumerate() {
                if let Some(area) = area_allocator.allocate(id, width, height) {
                    texture_atlas.texture.add_in_layer(
                        device,
                        physical_device,
                        layer_index as _,
                        area,
                        image_data,
                    );
                    return ShaderTextureData {
                        texture_index: texture_index as _,
                        layer_index: layer_index as _,
                        area: area.into(),
                        total_width: texture_atlas.get_texture().width() as _,
                        total_height: texture_atlas.get_texture().height() as _,
                        ..Default::default()
                    };
                }
            }
        }
        panic!("Unable to allocate texture")
    }

    pub fn get_texture_data(&self, id: &TextureId) -> Option<ShaderTextureData> {
        for (texture_index, texture_atlas) in self.texture_atlas.iter().enumerate() {
            for (layer_index, area_allocator) in texture_atlas.allocators.iter().enumerate() {
                if let Some(area) = area_allocator.get_area(id) {
                    return Some(ShaderTextureData {
                        texture_index: texture_index as _,
                        layer_index: layer_index as _,
                        area: (&area).into(),
                        total_width: texture_atlas.get_texture().width() as _,
                        total_height: texture_atlas.get_texture().height() as _,
                        ..Default::default()
                    });
                }
            }
        }
        None
    }
}
