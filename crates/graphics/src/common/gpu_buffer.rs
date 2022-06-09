use std::sync::atomic::AtomicBool;

use inox_resources::to_slice;

use crate::RenderCoreContext;

#[derive(Default)]
pub struct GpuBuffer {
    gpu_buffer: Option<wgpu::Buffer>,
    offset: u64,
    size: u64,
}

impl Drop for GpuBuffer {
    fn drop(&mut self) {
        self.release();
    }
}

impl GpuBuffer {
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
    pub fn overwrite_buffer<T>(&mut self, render_core_context: &RenderCoreContext, data: &[T]) {
        if data.is_empty() {
            return;
        }
        let data_size = data.len() as u64 * std::mem::size_of::<T>() as u64;
        debug_assert!(
            data_size <= self.size,
            "Trying to overwrite a buffer exceeding its size"
        );
        inox_profiler::scoped_profile!("DataBuffer::overwrite_buffer");

        if let Some(gpu_buffer) = self.gpu_buffer.as_mut() {
            {
                let slice = gpu_buffer.slice(0..self.size);
                let is_ready = std::sync::Arc::new(AtomicBool::new(false));
                let is_ready_clone = is_ready.clone();
                slice.map_async(wgpu::MapMode::Write, move |_v| {
                    is_ready_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                });
                render_core_context.device.poll(wgpu::Maintain::Wait);
                while !is_ready.load(std::sync::atomic::Ordering::SeqCst) {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                let mut view = slice.get_mapped_range_mut();
                let old_data = view.as_mut();
                let new_data = to_slice(data);
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        new_data.as_ptr(),
                        old_data.as_mut_ptr(),
                        new_data.len(),
                    );
                }
                self.size = data.len() as u64 * std::mem::size_of::<T>() as u64;
            }
            gpu_buffer.unmap();
        }
    }

    pub fn add_to_gpu_buffer<T>(&mut self, render_core_context: &RenderCoreContext, data: &[T]) {
        if data.is_empty() {
            return;
        }
        inox_profiler::scoped_profile!("DataBuffer::add_to_gpu_buffer");
        render_core_context.queue.write_buffer(
            self.gpu_buffer.as_ref().unwrap(),
            self.offset,
            to_slice(data),
        );
        self.offset += data.len() as u64 * std::mem::size_of::<T>() as u64;
    }
    pub fn read_from_gpu(&self, render_core_context: &RenderCoreContext) -> Option<Vec<u8>> {
        if self.size == 0 {
            return None;
        }
        inox_profiler::scoped_profile!("DataBuffer::read_from_gpu");
        self.gpu_buffer.as_ref().map(|gpu_buffer| {
            let result = {
                let slice = gpu_buffer.slice(0..self.size);
                let is_ready = std::sync::Arc::new(AtomicBool::new(false));
                let is_ready_clone = is_ready.clone();
                slice.map_async(wgpu::MapMode::Read, move |_v| {
                    is_ready_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                });
                render_core_context.device.poll(wgpu::Maintain::Wait);
                while !is_ready.load(std::sync::atomic::Ordering::SeqCst) {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                let view = slice.get_mapped_range();
                let data = view.as_ref();
                data.to_vec()
            };
            gpu_buffer.unmap();
            result
        })
    }
    pub fn read_from_gpu_as<T>(&self, render_core_context: &RenderCoreContext) -> Option<Vec<T>>
    where
        T: Sized + Clone,
    {
        self.read_from_gpu(render_core_context)
            .map(|data| to_slice(data.as_slice()).to_vec())
    }

    pub fn is_valid(&self) -> bool {
        self.gpu_buffer.is_some()
    }
    pub fn gpu_buffer(&self) -> Option<&wgpu::Buffer> {
        self.gpu_buffer.as_ref()
    }
}
