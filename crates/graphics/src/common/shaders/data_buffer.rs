use wgpu::util::DeviceExt;

use crate::{utils::to_u8_slice, RenderContext};

pub struct DataBuffer<T, const N: usize>
where
    T: Default + Copy,
{
    data: [T; N],
    data_buffer: wgpu::Buffer,
}

impl<T, const N: usize> DataBuffer<T, N>
where
    T: Default + Copy,
{
    pub fn new(context: &RenderContext) -> Self {
        let data = [T::default(); N];

        let typename = std::any::type_name::<T>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();
        let label = format!("{} Buffer", typename);
        let data_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label.as_str()),
                contents: to_u8_slice(&data),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            });

        Self { data, data_buffer }
    }

    pub fn send_to_gpu(&self, context: &RenderContext) {
        context
            .queue
            .write_buffer(&self.data_buffer, 0, to_u8_slice(&self.data));
    }

    pub fn data(&self) -> &[T] {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
    pub fn data_buffer(&self) -> &wgpu::Buffer {
        &self.data_buffer
    }
}
