use crate::{utils::to_u8_slice, RenderContext};

#[derive(Default)]
pub struct DataBuffer {
    gpu_buffer: Option<wgpu::Buffer>,
    offset: u64,
    size: u64,
}

impl DataBuffer {
    pub fn init<T>(&mut self, context: &RenderContext, size: u64, usage: wgpu::BufferUsages) {
        if !self.is_valid() || self.size != size {
            let typename = std::any::type_name::<T>()
                .split(':')
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .to_string();
            let label = format!("{} Buffer", typename);
            let data_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label.as_str()),
                size,
                mapped_at_creation: false,
                usage,
            });
            self.gpu_buffer = Some(data_buffer);
        }
        self.offset = 0;
        self.size = size;
    }
    pub fn add_to_gpu_buffer<T>(&mut self, context: &RenderContext, data: &[T]) {
        context.queue.write_buffer(
            self.gpu_buffer.as_ref().unwrap(),
            self.offset,
            to_u8_slice(data),
        );
        self.offset += data.len() as u64 * std::mem::size_of::<T>() as u64;
    }

    pub fn is_valid(&self) -> bool {
        self.gpu_buffer.is_some()
    }
    pub fn gpu_buffer(&self) -> &wgpu::Buffer {
        self.gpu_buffer.as_ref().unwrap()
    }
}
