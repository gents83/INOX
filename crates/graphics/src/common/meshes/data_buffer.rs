use std::any::type_name;

use sabi_resources::{Buffer, BufferData, ResourceId};
use wgpu::util::DeviceExt;

use crate::{utils::to_u8_slice, RenderContext};

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

impl<T, const U: u32> GpuBuffer<T, U>
where
    T: Clone,
{
    pub fn add(&mut self, id: &ResourceId, value: &[T]) -> u32 {
        let mut is_removed = false;
        if self.cpu_buffer.get(id).is_some() {
            self.remove(id);
            is_removed = true;
        }
        self.is_realloc_needed = self.cpu_buffer.allocate(id, value);
        self.is_realloc_needed |= is_removed;
        self.is_dirty = true;

        self.cpu_buffer.get(id).unwrap().start as _
    }
    pub fn update(&mut self, start_index: u32, value: &[T]) {
        self.cpu_buffer.update(start_index as _, value);
        self.is_dirty = true;
    }
    pub fn swap(&mut self, index: u32, other: u32) {
        self.cpu_buffer.swap(index as _, other as _);
    }
    pub fn get(&self, id: &ResourceId) -> Option<&BufferData> {
        self.cpu_buffer.get(id)
    }
    pub fn get_mut(&mut self, id: &ResourceId) -> Option<&mut [T]> {
        self.cpu_buffer.get_mut(id)
    }
    pub fn remove(&mut self, id: &ResourceId) {
        self.cpu_buffer.remove_with_id(id);
    }
    pub fn is_empty(&self) -> bool {
        self.cpu_buffer.is_empty()
    }
    pub fn clear(&mut self) {
        if let Some(buffer) = self.gpu_buffer.take() {
            buffer.destroy();
        }
        self.cpu_buffer.clear();
    }
    pub fn send_to_gpu(&mut self, context: &RenderContext) {
        if !self.is_dirty {
            return;
        }
        if self.is_realloc_needed {
            self.create_gpu_buffer(context);
        } else if let Some(buffer) = &self.gpu_buffer {
            let data = self.cpu_buffer.data();
            if data.is_empty() {
                return;
            }
            context.queue.write_buffer(buffer, 0, to_u8_slice(data));
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

    fn create_gpu_buffer(&mut self, context: &RenderContext) {
        if let Some(buffer) = self.gpu_buffer.take() {
            buffer.destroy();
        }
        let data = self.cpu_buffer.data();
        if data.is_empty() {
            self.gpu_buffer = None;
            self.is_realloc_needed = false;
            return;
        }
        let buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
    pub fn data(&self) -> &[T] {
        self.cpu_buffer.data()
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
