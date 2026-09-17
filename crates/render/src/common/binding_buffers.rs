use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{AsBinding, BufferId, BufferRef, RenderContext};

#[derive(Default)]
pub struct BindingDataBuffer {
    pub buffers: RwLock<HashMap<BufferId, BufferRef>>,
    changed_this_frame: RwLock<HashMap<BufferId, bool>>,
}

pub type BindingDataBufferRc = Arc<BindingDataBuffer>;

impl BindingDataBuffer {
    pub fn has_buffer(&self, uid: &BufferId) -> bool {
        self.buffers.read().unwrap().contains_key(uid)
    }
    pub fn is_buffer_changed(&self, id: BufferId) -> bool {
        if let Some(v) = self.changed_this_frame.read().unwrap().get(&id) {
            return *v;
        }
        !self.buffers.read().unwrap().contains_key(&id)
    }
    fn is_buffer_or_usage_changed(&self, id: BufferId, usage: wgpu::BufferUsages) -> bool {
        if self.is_buffer_changed(id) {
            return true;
        }
        let buffers = self.buffers.read().unwrap();
        let buffer = buffers.get(&id).unwrap();
        !buffer.usage().contains(usage)
    }
    pub fn mark_buffer_as_changed(&self, id: BufferId) {
        self.changed_this_frame.write().unwrap().insert(id, true);
    }
    pub fn clear_buffers_changed(&self) {
        self.changed_this_frame.write().unwrap().clear();
    }
    pub fn bind_buffer_with_id<T>(
        &self,
        id: BufferId,
        label: Option<&str>,
        data: &mut T,
        count: Option<usize>,
        usage: wgpu::BufferUsages,
        render_context: &RenderContext,
    ) -> bool
    where
        T: AsBinding,
    {
        let mut is_changed = self.is_buffer_or_usage_changed(id, usage);
        if is_changed {
            let mut bind_data_buffer = self.buffers.write().unwrap();
            let buffer = bind_data_buffer.entry(id).or_default();
            is_changed |= buffer.bind(label, data, count, usage, render_context);
            if is_changed {
                self.mark_buffer_as_changed(id);
            }
        }
        is_changed
    }
    pub fn bind_buffer<T>(
        &self,
        label: Option<&str>,
        data: &mut T,
        count: Option<usize>,
        usage: wgpu::BufferUsages,
        render_context: &RenderContext,
    ) -> bool
    where
        T: AsBinding,
    {
        let id = data.buffer_id();
        self.bind_buffer_with_id(id, label, data, count, usage, render_context)
    }

    /// Rewrite an already-bound storage buffer immediately after its CPU mirror changes.
    ///
    /// `mark_buffer_as_changed` only schedules a later binding pass. Queue headers are
    /// reset during pass updates, after binding initialization, so they need this direct
    /// upload path before the command buffer is encoded.
    pub fn update_buffer<T>(&self, data: &mut T, render_context: &RenderContext) -> bool
    where
        T: AsBinding,
    {
        let id = data.buffer_id();
        let usage = wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC;
        let mut buffers = self.buffers.write().unwrap();
        let buffer = buffers.entry(id).or_default();
        let changed = buffer.bind(None, data, None, usage, render_context);
        if changed {
            self.mark_buffer_as_changed(id);
        }
        changed
    }
}
