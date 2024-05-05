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
        usage != buffer.usage()
    }
    pub fn mark_buffer_as_changed(&self, id: BufferId) {
        self.changed_this_frame.write().unwrap().insert(id, true);
    }
    pub fn reset_buffers_changed(&self) {
        self.changed_this_frame.write().unwrap().clear();
    }
    pub fn bind_buffer<T>(
        &self,
        label: Option<&str>,
        data: &mut T,
        with_count: bool,
        usage: wgpu::BufferUsages,
        render_context: &RenderContext,
    ) -> bool
    where
        T: AsBinding,
    {
        let id = data.buffer_id();
        let mut is_changed = self.is_buffer_or_usage_changed(id, usage);
        if is_changed {
            let mut bind_data_buffer = self.buffers.write().unwrap();
            let buffer = bind_data_buffer.entry(id).or_default();
            is_changed |= buffer.bind(label, data, with_count, usage, render_context);
            self.changed_this_frame.write().unwrap().remove(&id);
        }
        is_changed
    }
}
