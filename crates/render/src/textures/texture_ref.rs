use std::ops::Deref;

use wgpu::util::{align_to, DeviceExt};

#[cfg(not(target_arch = "wasm32"))]
use crate::WebGpuContext;
use crate::{platform::has_multisampling_support, TextureFormat, TextureId};

use super::area::Area;

pub struct TextureView {
    view: wgpu::TextureView,
}

impl From<wgpu::TextureView> for TextureView {
    fn from(value: wgpu::TextureView) -> Self {
        Self { view: value }
    }
}
impl From<TextureView> for wgpu::TextureView {
    fn from(value: TextureView) -> Self {
        value.view
    }
}

impl AsRef<wgpu::TextureView> for TextureView {
    fn as_ref(&self) -> &wgpu::TextureView {
        &self.view
    }
}

impl Deref for TextureView {
    type Target = wgpu::TextureView;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

pub struct TextureRef {
    id: TextureId,
    texture: wgpu::Texture,
    views: Vec<TextureView>,
    width: u32,
    height: u32,
    layers_count: u32,
    sample_count: u32,
    format: TextureFormat,
    usage: wgpu::TextureUsages,
}

impl TextureRef {
    pub fn create(
        device: &wgpu::Device,
        id: TextureId,
        texture_params: (u32, u32, u32, u32, TextureFormat, wgpu::TextureUsages, u32),
    ) -> Self {
        let width = texture_params.0;
        let height = texture_params.1;
        let layers_count = texture_params.2;
        let sample_count = if has_multisampling_support() {
            texture_params.3
        } else {
            1
        };
        let format = texture_params.4;
        let usage = texture_params.5;
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: layers_count,
        };
        let mips_count = texture_params.6;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(format!("Texture[{id}]").as_str()),
            size,
            mip_level_count: mips_count,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: format.into(),
            usage,
            view_formats: &[format.into()],
        });
        let dimension = if layers_count > 1 {
            Some(wgpu::TextureViewDimension::D2Array)
        } else {
            Some(wgpu::TextureViewDimension::D2)
        };
        let mut texture = Self {
            id,
            texture,
            views: Vec::new(),
            width,
            height,
            layers_count,
            sample_count,
            format,
            usage,
        };
        //create_mips
        for _ in 0..mips_count {
            texture.add_view(format, Some(usage), dimension.unwrap(), layers_count);
        }
        texture
    }
    fn add_view(
        &mut self,
        format: TextureFormat,
        usage: Option<wgpu::TextureUsages>,
        dimension: wgpu::TextureViewDimension,
        layers_count: u32,
    ) {
        let view = self.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(format!("TextureView[{}]_mip[{}]", self.id, self.views.len()).as_str()),
            format: Some(format.into()),
            dimension: Some(dimension),
            aspect: format.aspect(),
            usage,
            base_mip_level: self.views.len() as _,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(layers_count),
            swizzle: wgpu::TextureComponentSwizzle::default(),
        });
        self.views.push(view.into());
    }
    pub fn view(&self, index: usize) -> &TextureView {
        &self.views[index]
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
    pub fn is_multisample(&self) -> bool {
        self.sample_count > 1
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
        let (block_width, block_height) = format.block_dimensions();
        let (area_width, area_height) = (area.width / block_width, area.height / block_height);
        let block_copy_size = format
            .block_copy_size(Some(self.format.aspect()))
            .unwrap_or(1);
        let original_bytes_per_row = area_width * block_copy_size;
        let bytes_per_row = align_to(original_bytes_per_row, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer_size = bytes_per_row * area_height; // should be multiplied for number of layers

        let mut padded_data = vec![0; buffer_size as _];
        let mut offset = 0;
        for row in 0..area_height as usize {
            padded_data[offset..offset + original_bytes_per_row as usize].copy_from_slice(
                &data[row * original_bytes_per_row as usize
                    ..(row + 1) * original_bytes_per_row as usize],
            );
            offset += bytes_per_row as usize;
        }
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("image staging buffer"),
            contents: &padded_data,
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        inox_profiler::gpu_scoped_profile!(encoder, device, "encoder::copy_buffer_to_texture");
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row as _),
                    rows_per_image: Some(area_height),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: area.x,
                    y: area.y,
                    z: layer_index,
                },
                aspect: self.format.aspect(),
            },
            wgpu::Extent3d {
                width: area_width,
                height: area_height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Read an Rgba32Float render target through a temporary staging buffer.
    ///
    /// This is intentionally native-only and bounded to the first mip/layer. It is used by
    /// opt-in image correctness smoke tests, never by the normal render loop.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read_rgba32f(&self, context: &WebGpuContext) -> Option<Vec<f32>> {
        if self.format != TextureFormat::Rgba32Float
            || self.width == 0
            || self.height == 0
            || !self.usage.contains(wgpu::TextureUsages::COPY_SRC)
        {
            return None;
        }
        let bytes_per_row = align_to(
            self.width as u64 * 16,
            wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64,
        );
        let staging_size = bytes_per_row * self.height as u64;
        let staging = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Native image smoke staging"),
            size: staging_size,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        });
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Native image smoke readback"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: self.format.aspect(),
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row as u32),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        context.queue.submit(std::iter::once(encoder.finish()));

        let slice = staging.slice(..);
        let result = std::sync::Arc::new(std::sync::Mutex::new(None));
        let result_clone = result.clone();
        slice.map_async(wgpu::MapMode::Read, move |value| {
            if let Ok(mut result) = result_clone.lock() {
                *result = Some(value);
            }
        });
        if context
            .device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(std::time::Duration::from_secs(5)),
            })
            .is_err()
        {
            staging.unmap();
            return None;
        }
        if !matches!(result.lock().ok()?.take(), Some(Ok(()))) {
            staging.unmap();
            return None;
        }
        let mapped = match slice.get_mapped_range() {
            Ok(mapped) => mapped,
            Err(_) => {
                staging.unmap();
                return None;
            }
        };
        let mut pixels = Vec::with_capacity(self.width as usize * self.height as usize * 4);
        for row in mapped
            .chunks_exact(bytes_per_row as usize)
            .take(self.height as usize)
        {
            for pixel in row[..self.width as usize * 16].as_chunks::<4>().0 {
                pixels.push(f32::from_ne_bytes([pixel[0], pixel[1], pixel[2], pixel[3]]));
            }
        }
        drop(mapped);
        staging.unmap();
        Some(pixels)
    }

    /// Read an R32Uint render target through a temporary staging buffer.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read_r32uint(&self, context: &WebGpuContext) -> Option<Vec<u32>> {
        if self.format != TextureFormat::R32Uint
            || self.width == 0
            || self.height == 0
            || !self.usage.contains(wgpu::TextureUsages::COPY_SRC)
        {
            return None;
        }
        let bytes_per_row = align_to(
            self.width as u64 * 4,
            wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64,
        );
        let staging = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Native visibility readback staging"),
            size: bytes_per_row * self.height as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        });
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Native visibility readback"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: self.format.aspect(),
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row as u32),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        context.queue.submit(std::iter::once(encoder.finish()));
        let slice = staging.slice(..);
        let result = std::sync::Arc::new(std::sync::Mutex::new(None));
        let result_clone = result.clone();
        slice.map_async(wgpu::MapMode::Read, move |value| {
            if let Ok(mut result) = result_clone.lock() {
                *result = Some(value);
            }
        });
        if context
            .device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(std::time::Duration::from_secs(5)),
            })
            .is_err()
            || !matches!(result.lock().ok()?.take(), Some(Ok(())))
        {
            staging.unmap();
            return None;
        }
        let mapped = match slice.get_mapped_range() {
            Ok(mapped) => mapped,
            Err(_) => {
                staging.unmap();
                return None;
            }
        };
        let mut values = Vec::with_capacity(self.width as usize * self.height as usize);
        for row in mapped
            .chunks_exact(bytes_per_row as usize)
            .take(self.height as usize)
        {
            for pixel in row[..self.width as usize * 4].as_chunks::<4>().0 {
                values.push(u32::from_ne_bytes([pixel[0], pixel[1], pixel[2], pixel[3]]));
            }
        }
        drop(mapped);
        staging.unmap();
        Some(values)
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32, mips_count: u32) {
        let old_width = self.width;
        let old_height = self.height;
        let old_mips_count = self.views.len() as u32;
        self.texture.destroy();
        self.views.clear();
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Resizes can replace a large target while the old allocation is still
            // deferred by the backend. Wait after destruction so a maximize or
            // repeated resize does not require old and new targets concurrently.
            let _ = device.poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(std::time::Duration::from_secs(5)),
            });
        }
        inox_log::debug_log!(
            "Resizing render target {} from {}x{} ({} mips) to {}x{} ({} mips)",
            self.id,
            old_width,
            old_height,
            old_mips_count,
            width,
            height,
            mips_count.max(1),
        );
        let replacement = Self::create(
            device,
            self.id,
            (
                width,
                height,
                self.layers_count,
                self.sample_count,
                self.format,
                self.usage,
                mips_count.max(1),
            ),
        );
        *self = replacement;
    }

    pub fn release(&mut self) {
        self.texture.destroy();
        self.width = 0;
        self.height = 0;
        self.layers_count = 0;
    }
}
