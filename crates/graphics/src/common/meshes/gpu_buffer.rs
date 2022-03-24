use std::any::type_name;

use inox_resources::{Buffer, BufferData, ResourceId};
use wgpu::util::DeviceExt;

use crate::utils::to_u8_slice;

pub struct GpuBuffer<T, const U: u32>
where
    T: Clone,
{
    cpu_buffer: Buffer<T>,
    gpu_buffer: Option<wgpu::Buffer>,
    is_dirty: bool,
    is_realloc_needed: bool,
}

impl<T, const U: u32> Default for GpuBuffer<T, U>
where
    T: Clone,
{
    fn default() -> Self {
        Self {
            cpu_buffer: Buffer::<T>::default(),
            gpu_buffer: None,
            is_dirty: false,
            is_realloc_needed: false,
        }
    }
}

impl<T, const U: u32> Clone for GpuBuffer<T, U>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            cpu_buffer: self.cpu_buffer.clone(),
            gpu_buffer: None,
            is_dirty: true,
            is_realloc_needed: true,
        }
    }
}

impl<T, const U: u32> GpuBuffer<T, U>
where
    T: Clone,
{
    pub fn add(&mut self, id: &ResourceId, value: &[T]) -> u32 {
        if self.cpu_buffer.get(id).is_some() {
            self.remove(id);
        }
        self.is_realloc_needed |= self.cpu_buffer.allocate(id, value);
        self.is_dirty = true;

        self.cpu_buffer.get(id).unwrap().start as _
    }
    pub fn update(&mut self, start_index: u32, value: &[T]) {
        self.cpu_buffer.update(start_index as _, value);
        self.is_dirty = true;
    }
    pub fn swap(&mut self, index: u32, other: u32) {
        if self.cpu_buffer.swap(index as _, other as _) {
            self.is_dirty = true;
        }
    }
    pub fn get(&self, id: &ResourceId) -> Option<&BufferData> {
        self.cpu_buffer.get(id)
    }
    pub fn get_mut(&mut self, id: &ResourceId) -> Option<&mut [T]> {
        self.cpu_buffer.get_mut(id)
    }
    pub fn remove(&mut self, id: &ResourceId) {
        if self.cpu_buffer.remove_with_id(id) {
            self.is_dirty = true;
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
            let data = self.cpu_buffer.total_data();
            if data.is_empty() {
                return;
            }
            inox_profiler::scoped_profile!("gpu_buffer::send_to_gpu - write_buffer");
            queue.write_buffer(buffer, 0, to_u8_slice(data));
        }
        self.is_dirty = false;
    }

    fn label(&self) -> &str {
        type_name::<T>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
    }

    fn create_gpu_buffer(&mut self, device: &wgpu::Device) {
        inox_profiler::scoped_profile!("gpu_buffer::create_gpu_buffer");
        if let Some(buffer) = self.gpu_buffer.take() {
            inox_profiler::scoped_profile!("gpu_buffer::destroy_buffer");
            buffer.destroy();
        }
        let data = self.cpu_buffer.total_data();
        if data.is_empty() {
            self.gpu_buffer = None;
            self.is_realloc_needed = false;
            return;
        }
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(self.label()),
            contents: to_u8_slice(data),
            usage: wgpu::BufferUsages::from_bits(U).unwrap() | wgpu::BufferUsages::COPY_DST,
        });
        self.gpu_buffer = Some(buffer);
        self.is_realloc_needed = false;
    }

    pub fn len(&self) -> usize {
        self.cpu_buffer.len()
    }
    pub fn for_each_data<F>(&self, f: F)
    where
        F: FnMut(usize, &T),
    {
        self.cpu_buffer.for_each_data(f);
    }
    pub fn data_at_index(&self, index: u32) -> &T {
        self.cpu_buffer.data_at_index(index as _)
    }
    pub fn gpu_buffer(&self) -> Option<&wgpu::Buffer> {
        self.gpu_buffer.as_ref()
    }
}

impl<T, const U: u32> Drop for GpuBuffer<T, U>
where
    T: Clone,
{
    fn drop(&mut self) {
        self.clear();
    }
}
