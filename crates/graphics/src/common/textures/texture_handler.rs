use std::sync::{Arc, RwLock, RwLockReadGuard};

use inox_log::debug_log;

use crate::{TextureBlock, TextureFormat, TextureId, TextureInfo, TextureUsage};

use super::{gpu_texture::GpuTexture, texture_atlas::TextureAtlas};

pub const DEBUG_TEXTURES: bool = false;

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
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let unfiltered_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let depth_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::Less),
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
                if DEBUG_TEXTURES {
                    debug_log!(
                        "Removing texture atlas {} with format {:?}",
                        atlas.texture_id(),
                        atlas.texture_format()
                    );
                }
            }
            !atlas.is_empty()
        });
        self.render_targets.write().unwrap().retain_mut(|t| {
            if t.id() == id {
                t.release();
                if DEBUG_TEXTURES {
                    debug_log!("Removing render target {} with format {:?}", id, t.format());
                }
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
        sample_count: u32,
    ) -> usize {
        let texture = GpuTexture::create(
            device,
            *id,
            (
                dimensions.0,
                dimensions.1,
                1,
                sample_count,
                format,
                usage.into(),
            ),
        );
        if DEBUG_TEXTURES {
            inox_log::debug_log!(
                "Adding new render target {} {:?}x{:?} with format {:?}",
                id,
                dimensions.0,
                dimensions.1,
                format
            );
        }
        self.render_targets.write().unwrap().push(texture);
        self.render_targets.read().unwrap().len() - 1
    }

    pub fn add_image_to_texture_atlas(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        id: &TextureId,
        texture_params: (u32, u32, TextureFormat, u32, bool),
        image_data: &[u8],
    ) -> TextureInfo {
        for (texture_index, texture_atlas) in
            self.texture_atlas.write().unwrap().iter_mut().enumerate()
        {
            let mut atlas_index = texture_index as i32;
            if texture_params.4 {
                // isLUT
                atlas_index *= -1;
            }
            if texture_atlas.texture_format() == &texture_params.2 {
                if let Some(texture_data) = texture_atlas.allocate(
                    device,
                    encoder,
                    id,
                    atlas_index,
                    (texture_params.0, texture_params.1),
                    image_data,
                ) {
                    if DEBUG_TEXTURES {
                        inox_log::debug_log!(
                            "Adding image {} {:?}x{:?} into atlas {} at layer {}",
                            id,
                            texture_params.0,
                            texture_params.1,
                            texture_index,
                            texture_data.layer_index
                        );
                    }
                    return texture_data;
                }
            }
        }
        self.texture_atlas
            .write()
            .unwrap()
            .push(TextureAtlas::create_default(
                device,
                texture_params.2,
                texture_params.3,
            ));
        if DEBUG_TEXTURES {
            inox_log::debug_log!(
                "Adding new texture atlas {} at index {} with format {:?}",
                self.texture_atlas
                    .read()
                    .unwrap()
                    .last()
                    .unwrap()
                    .texture_id(),
                self.texture_atlas.read().unwrap().len() - 1,
                texture_params.2
            );
        }
        self.add_image_to_texture_atlas(device, encoder, id, texture_params, image_data)
    }

    pub fn update_texture_atlas(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        id: &TextureId,
        texture_block: &TextureBlock,
    ) {
        for (texture_index, texture_atlas) in
            self.texture_atlas.write().unwrap().iter_mut().enumerate()
        {
            if let Some(texture_info) = texture_atlas.texture_info(texture_index as _, id) {
                if DEBUG_TEXTURES {
                    inox_log::debug_log!(
                        "Updating block ({},{}) with size {:?}x{:?} in image {} {:?}x{:?} into atlas {} at layer {}",
                        texture_block.x,
                        texture_block.y,
                        texture_block.width,
                        texture_block.height,
                        id,
                        texture_info.width(),
                        texture_info.height(),
                        texture_index,
                        texture_info.layer_index
                    );
                }
                texture_atlas.update(
                    device,
                    encoder,
                    id,
                    texture_info.layer_index as _,
                    texture_block,
                );
            }
        }
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
