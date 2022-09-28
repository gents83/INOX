use std::num::NonZeroU32;

use wgpu::util::DeviceExt;

use crate::{TextureFormat, TextureId};

use super::area::Area;

pub struct Texture {
    id: TextureId,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    layers_count: u32,
    format: TextureFormat,
}

impl Texture {
    pub fn create(
        device: &wgpu::Device,
        id: TextureId,
        width: u32,
        height: u32,
        layers_count: u32,
        format: TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: layers_count,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(format!("Texture[{}]", id).as_str()),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format.into(),
            usage,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            id,
            texture,
            view,
            width,
            height,
            layers_count,
            format,
        }
    }
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
    pub fn format(&self) -> &TextureFormat {
        &self.format
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
    pub fn layers_count(&self) -> u32 {
        self.layers_count
    }
    pub fn send_to_gpu(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        layer_index: u32,
        area: &Area,
        data: &[u8],
    ) {
        // It is a webgpu requirement that:
        //   BufferCopyView.layout.bytes_per_row % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT == 0
        // So we calculate padded_width by rounding width up to the next
        // multiple of wgpu::COPY_BYTES_PER_ROW_ALIGNMENT.
        let format: wgpu::TextureFormat = self.format.into();
        let pixel_size = format.describe().block_size as u32;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padding = (align - (pixel_size * area.width) % align) % align;
        let padded_width = (pixel_size * area.width + padding) as usize;
        let padded_data_size = (pixel_size * area.width * area.height) as usize;

        let mut padded_data = vec![0; padded_data_size];

        for row in 0..area.height as usize {
            let offset = row * padded_width;

            padded_data[offset..offset + (pixel_size * area.width) as usize].copy_from_slice(
                &data[row * (pixel_size * area.width) as usize
                    ..(row + 1) * (pixel_size * area.width) as usize],
            )
        }
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("image staging buffer"),
            contents: &padded_data,
            usage: wgpu::BufferUsages::COPY_SRC,
        });
        let extent = wgpu::Extent3d {
            width: area.width,
            height: area.height,
            depth_or_array_layers: 1,
        };

        inox_profiler::gpu_scoped_profile!(encoder, device, "encoder::copy_buffer_to_texture");
        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(pixel_size * area.width + padding),
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

    pub fn release(&mut self) {
        self.texture.destroy();
        self.width = 0;
        self.height = 0;
        self.layers_count = 0;
    }
}
