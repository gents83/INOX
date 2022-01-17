use std::num::NonZeroU32;

use wgpu::util::DeviceExt;

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
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
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

    pub fn send_to_gpu(
        &self,
        context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        layer_index: u32,
        area: &Area,
        data: &[u8],
    ) {
        // It is a webgpu requirement that:
        //   BufferCopyView.layout.bytes_per_row % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT == 0
        // So we calculate padded_width by rounding width up to the next
        // multiple of wgpu::COPY_BYTES_PER_ROW_ALIGNMENT.
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padding = (align - (4 * area.width) % align) % align;
        let padded_width = (4 * area.width + padding) as usize;
        let padded_data_size = padded_width * area.height as usize;

        let mut padded_data = vec![0; padded_data_size];

        for row in 0..area.height as usize {
            let offset = row * padded_width;

            padded_data[offset..offset + 4 * area.width as usize].copy_from_slice(
                &data[row * 4 * area.width as usize..(row + 1) * 4 * area.width as usize],
            )
        }
        let buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("image staging buffer"),
                contents: &padded_data,
                usage: wgpu::BufferUsages::COPY_SRC,
            });
        let extent = wgpu::Extent3d {
            width: area.width,
            height: area.height,
            depth_or_array_layers: 1,
        };
        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(4 * area.width + padding),
                    rows_per_image: NonZeroU32::new(area.height),
                },
            },
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: area.x,
                    y: area.y,
                    z: layer_index as u32,
                },
                aspect: wgpu::TextureAspect::default(),
            },
            extent,
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
