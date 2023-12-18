use std::sync::{Arc, RwLock, RwLockReadGuard};

use inox_log::debug_log;

use crate::{TextureFormat, TextureId, TextureInfo, TextureUsage};

use super::{gpu_texture::GpuTexture, texture_atlas::TextureAtlas};

#[derive(Debug, Default, Clone, Copy)]
pub enum SamplerType {
    #[default]
    Default = 0,
    Unfiltered = 1,
    Depth = 2,
    Count = 3,
}

pub struct TextureHandler {
    texture_atlas: RwLock<Vec<TextureAtlas>>,
    render_targets: RwLock<Vec<GpuTexture>>,
    samplers: [wgpu::Sampler; SamplerType::Count as _],
}

pub type TextureHandlerRc = Arc<TextureHandler>;

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
        Self {
            texture_atlas: RwLock::new(Vec::new()),
            render_targets: RwLock::new(Vec::new()),
            samplers: [default_sampler, unfiltered_sampler, depth_sampler],
        }
    }
    pub fn sampler(&self, t: SamplerType) -> &wgpu::Sampler {
        &self.samplers[t as usize]
    }

    pub fn textures_atlas(&self) -> RwLockReadGuard<Vec<TextureAtlas>> {
        self.texture_atlas.read().unwrap()
    }
    pub fn render_targets(&self) -> RwLockReadGuard<Vec<GpuTexture>> {
        self.render_targets.read().unwrap()
    }

    pub fn texture_atlas_id(&self, index: usize) -> TextureId {
        *self.texture_atlas.read().unwrap()[index].texture_id()
    }

    pub fn remove(&self, id: &TextureId) {
        self.texture_atlas.write().unwrap().retain_mut(|atlas| {
            if atlas.remove(id) {
                atlas.destroy();
                debug_log!(
                    "Removing texture atlas with format {:?}",
                    atlas.texture_format()
                );
            }
            !atlas.is_empty()
        });
        self.render_targets.write().unwrap().retain_mut(|t| {
            if t.id() == id {
                t.release();
                debug_log!("Removing render target with format {:?}", t.format());
                return false;
            }
            true
        });
    }

    pub fn add_render_target(
        &self,
        device: &wgpu::Device,
        id: &TextureId,
        dimensions: (u32, u32),
        format: TextureFormat,
        usage: TextureUsage,
    ) -> usize {
        let texture = GpuTexture::create(
            device,
            *id,
            dimensions.0,
            dimensions.1,
            1,
            format,
            usage.into(),
        );
        inox_log::debug_log!(
            "Adding new render target {:?}x{:?} with format {:?}",
            dimensions.0,
            dimensions.1,
            format
        );
        self.render_targets.write().unwrap().push(texture);
        self.render_targets.read().unwrap().len() - 1
    }

    pub fn add_image_to_texture_atlas(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        id: &TextureId,
        dimensions: (u32, u32),
        format: TextureFormat,
        image_data: &[u8],
    ) -> TextureInfo {
        for (texture_index, texture_atlas) in
            self.texture_atlas.write().unwrap().iter_mut().enumerate()
        {
            if texture_atlas.texture_format() == &format {
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
        }
        self.texture_atlas
            .write()
            .unwrap()
            .push(TextureAtlas::create_default(device, format));
        inox_log::debug_log!("Adding new texture atlas with format {:?}", format);
        self.add_image_to_texture_atlas(device, encoder, id, dimensions, format, image_data)
    }

    pub fn texture_info(&self, id: &TextureId) -> Option<TextureInfo> {
        for (texture_index, texture_atlas) in self.texture_atlas.read().unwrap().iter().enumerate()
        {
            if let Some(texture_data) = texture_atlas.texture_info(texture_index as _, id) {
                return Some(texture_data);
            }
        }
        None
    }
}
