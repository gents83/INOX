use std::path::Path;

use inox_log::debug_log;

use crate::{TextureId, TextureInfo};

use super::texture_atlas::TextureAtlas;

pub struct TextureHandler {
    texture_atlas: Vec<TextureAtlas>,
    default_sampler: wgpu::Sampler,
    unfiltered_sampler: wgpu::Sampler,
    depth_sampler: wgpu::Sampler,
}

impl TextureHandler {
    pub fn create(device: &wgpu::Device) -> Self {
        let default_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let unfiltered_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let depth_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let texture_atlas = vec![TextureAtlas::create_default(device)];
        Self {
            texture_atlas,
            default_sampler,
            unfiltered_sampler,
            depth_sampler,
        }
    }
    pub fn default_sampler(&self) -> &wgpu::Sampler {
        &self.default_sampler
    }
    pub fn unfiltered_sampler(&self) -> &wgpu::Sampler {
        &self.unfiltered_sampler
    }
    pub fn depth_sampler(&self) -> &wgpu::Sampler {
        &self.depth_sampler
    }

    pub fn add_custom_texture(
        &mut self,
        device: &wgpu::Device,
        id: &TextureId,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) {
        self.texture_atlas.push(TextureAtlas::create_texture(
            device, id, width, height, 1, format, usage,
        ));
    }

    pub fn textures_atlas(&self) -> &[TextureAtlas] {
        self.texture_atlas.as_slice()
    }

    pub fn texture_index(&self, id: &TextureId) -> Option<usize> {
        for (texture_index, texture_atlas) in self.texture_atlas.iter().enumerate() {
            if texture_atlas
                .get_texture_data(texture_index as _, id)
                .is_some()
            {
                return Some(texture_index);
            }
        }
        None
    }

    pub fn get_texture_atlas(&self, id: &TextureId) -> Option<&TextureAtlas> {
        if let Some(index) = self.texture_atlas.iter().position(|t| t.texture_id() == id) {
            return Some(&self.texture_atlas[index]);
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.texture_atlas.is_empty()
    }

    pub fn copy(&self, device: &wgpu::Device, id: &TextureId, _image_data: &mut [u8]) {
        inox_profiler::scoped_profile!("texture::copy");

        self.texture_atlas.iter().for_each(|atlas| {
            if atlas.read_from_gpu(device, id) {
                todo!();
            }
        });
    }

    pub fn remove(&mut self, id: &TextureId) {
        let mut texture_to_remove = Vec::new();
        self.texture_atlas
            .iter_mut()
            .enumerate()
            .for_each(|(texture_index, atlas)| {
                if atlas.remove(texture_index as _, id) {
                    texture_to_remove.push(texture_index);
                }
            });
        texture_to_remove.iter().rev().for_each(|i| {
            let mut texture_atlas = self.texture_atlas.remove(*i);
            texture_atlas.destroy();
            debug_log!("Removing texture atlas {:?}", i);
        });
    }

    pub fn add_from_path(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        id: &TextureId,
        filepath: &Path,
    ) -> TextureInfo {
        let image = image::open(filepath).unwrap();
        self.add_image(
            device,
            encoder,
            id,
            (image.width(), image.height()),
            image.to_rgba8().as_raw().as_slice(),
        )
    }

    pub fn add_image(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        id: &TextureId,
        dimensions: (u32, u32),
        image_data: &[u8],
    ) -> TextureInfo {
        for (texture_index, texture_atlas) in self.texture_atlas.iter_mut().enumerate() {
            if let Some(texture_data) = texture_atlas.allocate(
                device,
                encoder,
                id,
                texture_index as _,
                dimensions,
                image_data,
            ) {
                return texture_data;
            }
        }
        self.texture_atlas
            .push(TextureAtlas::create_default(device));
        inox_log::debug_log!("Adding new texture atlas");
        self.add_image(device, encoder, id, dimensions, image_data)
    }

    pub fn get_texture_data(&self, id: &TextureId) -> Option<TextureInfo> {
        for (texture_index, texture_atlas) in self.texture_atlas.iter().enumerate() {
            if let Some(texture_data) = texture_atlas.get_texture_data(texture_index as _, id) {
                return Some(texture_data);
            }
        }
        None
    }
}
