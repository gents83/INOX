use std::num::NonZeroU32;

use crate::{RenderContext, TextureId};

use super::area::Area;

pub struct Texture {
    id: TextureId,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    width: u32,
    height: u32,
    layers_count: u32,
}

impl Texture {
    pub fn create(
        context: &RenderContext,
        id: TextureId,
        width: u32,
        height: u32,
        layers_count: u32,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: layers_count,
        };
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(format!("Texture[{}]", id).as_str()),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            id,
            texture,
            view,
            sampler,
            width,
            height,
            layers_count,
        }
    }

    pub fn id(&self) -> &TextureId {
        &self.id
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn write_to_gpu(
        &self,
        context: &RenderContext,
        layer_index: u32,
        area: &Area,
        data: &[u8],
    ) {
        let size = wgpu::Extent3d {
            width: area.width,
            height: area.height,
            depth_or_array_layers: layer_index,
        };
        context.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * area.width),
                rows_per_image: NonZeroU32::new(area.height),
            },
            size,
        );
    }

    pub fn read_from_gpu(&self, _context: &RenderContext, _area: &Area, _layer_index: u32) {
        todo!();
    }

    pub fn release(&mut self) {
        self.texture.destroy();
        self.width = 0;
        self.height = 0;
        self.layers_count = 0;
    }
}
