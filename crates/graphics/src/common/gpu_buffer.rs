use std::ops::Range;

use inox_resources::{to_u8_slice, Buffer, BufferData, ResourceId};
use wgpu::util::DeviceExt;

pub struct GpuBuffer<const U: u32> {
    cpu_buffer: Buffer,
    gpu_buffer: Option<wgpu::Buffer>,
    is_dirty: bool,
    is_realloc_needed: bool,
}

impl<const U: u32> Default for GpuBuffer<U> {
    fn default() -> Self {
        Self {
            cpu_buffer: Buffer::default(),
            gpu_buffer: None,
            is_dirty: false,
            is_realloc_needed: false,
        }
    }
}

impl<const U: u32> GpuBuffer<U> {
    pub fn add_with_size(
        &mut self,
        id: &ResourceId,
        value: &[u8],
        item_size: usize,
    ) -> Range<usize> {
        if self.cpu_buffer.get(id).is_some() {
            self.remove(id);
        }
        self.is_realloc_needed |= self.cpu_buffer.allocate_with_size(id, value, item_size);
        self.is_dirty = true;

        self.cpu_buffer.get(id).unwrap().item_range()
    }
    pub fn add<T>(&mut self, id: &ResourceId, value: &[T]) -> Range<usize> {
        if self.cpu_buffer.get(id).is_some() {
            self.remove(id);
        }
        self.is_realloc_needed |= self.cpu_buffer.allocate(id, value);
        self.is_dirty = true;

        self.cpu_buffer.get(id).unwrap().item_range()
    }
    pub fn update<T>(&mut self, start_index: u32, data: &[T]) {
        let data = to_u8_slice(data);
        self.cpu_buffer.update(start_index as _, data);
        self.is_dirty = true;
    }
    pub fn get(&self, id: &ResourceId) -> Option<&BufferData> {
        self.cpu_buffer.get(id)
    }
    pub fn remove(&mut self, id: &ResourceId) {
        if self.cpu_buffer.remove_with_id(id) {
            self.is_dirty = true;
        }

        if self.cpu_buffer.is_empty() {
            self.clear();
        }
    }
    pub fn is_empty(&self) -> bool {
        self.cpu_buffer.is_empty()
    }
    pub fn clear(&mut self) {
        if let Some(buffer) = self.gpu_buffer.take() {
            buffer.destroy();
        }
        self.cpu_buffer.clear();
        self.is_dirty = true;
        self.is_realloc_needed = true;
    }
    pub fn send_to_gpu(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if !self.is_dirty {
            return;
        }
        if self.is_realloc_needed || self.gpu_buffer.is_none() {
            self.create_gpu_buffer(device);
        } else if let Some(buffer) = &self.gpu_buffer {
            let data = self.cpu_buffer.data();
            if data.is_empty() {
                return;
            }
            inox_profiler::scoped_profile!("gpu_buffer::send_to_gpu - write_buffer");
            queue.write_buffer(buffer, 0, data);
        }
        self.is_dirty = false;
    }

    fn create_gpu_buffer(&mut self, device: &wgpu::Device) {
        inox_profiler::scoped_profile!("gpu_buffer::create_gpu_buffer");
        if let Some(buffer) = self.gpu_buffer.take() {
            inox_profiler::scoped_profile!("gpu_buffer::destroy_buffer");
            buffer.destroy();
        }
        let data = self.cpu_buffer.data();
        if data.is_empty() {
            self.gpu_buffer = None;
            self.is_realloc_needed = false;
            return;
        }
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("GpuBuffer {}", U).as_str()),
            contents: data,
            usage: wgpu::BufferUsages::from_bits(U).unwrap() | wgpu::BufferUsages::COPY_DST,
        });
        self.gpu_buffer = Some(buffer);
        self.is_realloc_needed = false;
    }

    pub fn len(&self) -> usize {
        self.cpu_buffer.item_count()
    }
    pub fn for_each_occupied<F>(&self, f: &mut F)
    where
        F: FnMut(&ResourceId, &Range<usize>),
    {
        self.cpu_buffer.for_each_occupied(f);
    }
    pub fn for_each_free<F>(&self, f: &mut F)
    where
        F: FnMut(&ResourceId, &Range<usize>),
    {
        self.cpu_buffer.for_each_free(f);
    }
    pub fn for_each_data<F, T>(&self, f: F)
    where
        F: FnMut(usize, &ResourceId, &T),
    {
        self.cpu_buffer.for_each_data(f);
    }
    pub fn gpu_buffer(&self) -> Option<&wgpu::Buffer> {
        self.gpu_buffer.as_ref()
    }
}

impl<const U: u32> Drop for GpuBuffer<U> {
    fn drop(&mut self) {
        self.clear();
    }
}
