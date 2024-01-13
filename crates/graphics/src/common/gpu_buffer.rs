use std::sync::atomic::AtomicBool;

use inox_resources::to_slice;

use crate::{AsBinding, RenderContext, WebGpuContext};

pub struct GpuBuffer {
    gpu_buffer: Option<wgpu::Buffer>,
    usage: wgpu::BufferUsages,
    offset: u64,
    size: u64,
    name: String,
}

impl Default for GpuBuffer {
    fn default() -> Self {
        Self {
            gpu_buffer: None,
            usage: wgpu::BufferUsages::empty(),
            offset: 0,
            size: 0,
            name: String::new(),
        }
    }
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
    pub fn usage(&self) -> wgpu::BufferUsages {
        self.usage
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn release(&mut self) {
        if let Some(buffer) = self.gpu_buffer.take() {
            buffer.destroy();
        }
    }
    fn init(
        &mut self,
        render_context: &RenderContext,
        size: u64,
        usage: wgpu::BufferUsages,
        buffer_name: &str,
    ) -> bool {
        inox_profiler::scoped_profile!("GpuBuffer::init({})", buffer_name);

        self.offset = 0;
        if size > self.size || usage != self.usage {
            let label = format!("{buffer_name} Buffer");
            self.name = label;
            self.release();
            self.usage |= usage;
            let data_buffer = render_context
                .webgpu
                .device
                .create_buffer(&wgpu::BufferDescriptor {
                    label: Some(self.name.as_str()),
                    size,
                    mapped_at_creation: false,
                    usage: self.usage,
                });
            self.gpu_buffer = Some(data_buffer);
            self.size = size;
            return true;
        }
        false
    }
    pub fn init_from_type<T>(
        &mut self,
        render_context: &RenderContext,
        size: u64,
        usage: wgpu::BufferUsages,
    ) -> bool {
        let typename = std::any::type_name::<T>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();
        self.init(render_context, size, usage, typename.as_str())
    }
    pub fn overwrite_buffer<T>(&mut self, render_core_context: &WebGpuContext, data: &[T]) {
        if data.is_empty() {
            return;
        }
        let data_size = data.len() as u64 * std::mem::size_of::<T>() as u64;
        debug_assert!(
            data_size <= self.size,
            "Trying to overwrite a buffer exceeding its size"
        );
        inox_profiler::scoped_profile!("GpuBuffer::overwrite_buffer");

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
    pub fn add_to_gpu_buffer<T>(&mut self, render_context: &RenderContext, data: &[T]) {
        if data.is_empty() {
            return;
        }
        inox_profiler::scoped_profile!("GpuBuffer::add_to_gpu_buffer({})", &self.name);
        render_context.webgpu.queue.write_buffer(
            self.gpu_buffer.as_ref().unwrap(),
            self.offset,
            to_slice(data),
        );
        self.offset += data.len() as u64 * std::mem::size_of::<T>() as u64;
    }

    pub fn bind<T>(
        &mut self,
        label: Option<&str>,
        data: &mut T,
        usage: wgpu::BufferUsages,
        render_context: &RenderContext,
    ) -> bool
    where
        T: AsBinding,
    {
        inox_profiler::scoped_profile!("GpuBuffer::bind({})", &self.name);
        let name = if let Some(name) = label {
            name.to_string()
        } else {
            let id = data.id();
            format!("{}[{}]", std::any::type_name::<T>(), id)
        };
        let is_changed = self.init(render_context, data.size(), usage, name.as_str());
        if usage.intersects(wgpu::BufferUsages::COPY_DST) {
            data.fill_buffer(render_context, self);
            data.set_dirty(false);
        }
        is_changed
    }
    pub fn read_from_gpu(&self, render_core_context: &WebGpuContext) -> Option<Vec<u8>> {
        if self.size == 0 {
            return None;
        }
        inox_profiler::scoped_profile!("GpuBuffer::read_from_gpu");
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
    pub fn read_from_gpu_as<T>(&self, render_core_context: &WebGpuContext) -> Option<Vec<T>>
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
