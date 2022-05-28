use inox_resources::to_u8_slice;

use crate::RenderCoreContext;

#[derive(Default)]
pub struct DataBuffer {
    gpu_buffer: Option<wgpu::Buffer>,
    offset: u64,
    size: u64,
}

impl Drop for DataBuffer {
    fn drop(&mut self) {
        self.release();
    }
}

impl DataBuffer {
    pub fn size(&self) -> u64 {
        self.size
    }
    pub fn release(&mut self) {
        if let Some(buffer) = self.gpu_buffer.take() {
            buffer.destroy();
        }
    }
    pub fn init(
        &mut self,
        render_core_context: &RenderCoreContext,
        size: u64,
        usage: wgpu::BufferUsages,
        buffer_name: &str,
    ) -> bool {
        inox_profiler::scoped_profile!("DataBuffer::init");

        self.offset = 0;
        if !self.is_valid() || self.size != size {
            let label = format!("{} Buffer", buffer_name);
            self.release();
            let data_buffer = render_core_context
                .device
                .create_buffer(&wgpu::BufferDescriptor {
                    label: Some(label.as_str()),
                    size,
                    mapped_at_creation: false,
                    usage,
                });
            self.gpu_buffer = Some(data_buffer);
            self.size = size;
            return true;
        }
        false
    }
    pub fn init_from_type<T>(
        &mut self,
        render_core_context: &RenderCoreContext,
        size: u64,
        usage: wgpu::BufferUsages,
    ) -> bool {
        let typename = std::any::type_name::<T>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();
        self.init(render_core_context, size, usage, typename.as_str())
    }
    pub fn add_to_gpu_buffer<T>(&mut self, render_core_context: &RenderCoreContext, data: &[T]) {
        if data.is_empty() {
            return;
        }
        inox_profiler::scoped_profile!("DataBuffer::add_to_gpu_buffer");
        render_core_context.queue.write_buffer(
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
