use inox_resources::to_u8_slice;

use crate::RenderContext;

#[derive(Default)]
pub struct DataBuffer {
    gpu_buffer: Option<wgpu::Buffer>,
    offset: u64,
    size: u64,
}

impl DataBuffer {
    pub fn init(
        &mut self,
        context: &RenderContext,
        size: u64,
        usage: wgpu::BufferUsages,
        buffer_name: &str,
    ) {
        inox_profiler::scoped_profile!("DataBuffer::init");

        if !self.is_valid() || self.size != size {
            let label = format!("{} Buffer", buffer_name);
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
    pub fn init_from_type<T>(
        &mut self,
        context: &RenderContext,
        size: u64,
        usage: wgpu::BufferUsages,
    ) {
        let typename = std::any::type_name::<T>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();
        self.init(context, size, usage, typename.as_str());
    }
    pub fn add_to_gpu_buffer<T>(&mut self, context: &RenderContext, data: &[T]) {
        inox_profiler::scoped_profile!("DataBuffer::add_to_gpu_buffer");
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
    pub fn gpu_buffer(&self) -> Option<&wgpu::Buffer> {
        self.gpu_buffer.as_ref()
    }
}
