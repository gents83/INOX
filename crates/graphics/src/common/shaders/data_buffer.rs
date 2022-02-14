use wgpu::util::DeviceExt;

use crate::{utils::to_u8_slice, RenderContext};

pub struct DataBuffer<T, const N: usize>
where
    T: Default + Copy,
{
    data: [T; N],
    data_buffer: Option<wgpu::Buffer>,
}
impl<T, const N: usize> Default for DataBuffer<T, N>
where
    T: Default + Copy,
{
    fn default() -> Self {
        let data = [T::default(); N];

        Self {
            data,
            data_buffer: None,
        }
    }
}

impl<T, const N: usize> DataBuffer<T, N>
where
    T: Default + Copy,
{
    pub fn send_to_gpu(&mut self, context: &RenderContext) {
        if self.data_buffer.is_none() {
            let typename = std::any::type_name::<T>()
                .split(':')
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .to_string();
            let label = format!("{} Buffer", typename);
            let data_buffer =
                context
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(label.as_str()),
                        contents: to_u8_slice(&self.data),
                        usage: wgpu::BufferUsages::UNIFORM
                            | wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_DST,
                    });
            self.data_buffer = Some(data_buffer);
        }
        context.queue.write_buffer(
            self.data_buffer.as_ref().unwrap(),
            0,
            to_u8_slice(&self.data),
        );
    }

    pub fn data(&self) -> &[T] {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
    pub fn is_valid(&self) -> bool {
        self.data_buffer.is_some()
    }
    pub fn data_buffer(&self) -> &wgpu::Buffer {
        self.data_buffer.as_ref().unwrap()
    }
}
