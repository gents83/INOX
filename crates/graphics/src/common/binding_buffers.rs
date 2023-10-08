use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{AsBinding, BufferId, GpuBuffer, RenderCoreContext};

#[derive(Default)]
pub struct BindingDataBuffer {
    pub buffers: RwLock<HashMap<BufferId, GpuBuffer>>,
    changed_this_frame: RwLock<Vec<BufferId>>,
}

pub type BindingDataBufferRc = Arc<BindingDataBuffer>;

impl BindingDataBuffer {
    pub fn has_buffer(&self, uid: &BufferId) -> bool {
        self.buffers.read().unwrap().contains_key(uid)
    }
    pub fn is_changed(&self, uid: &BufferId) -> bool {
        self.changed_this_frame
            .read()
            .unwrap()
            .iter()
            .any(|id| id == uid)
    }
    pub fn reset_buffers_changed(&self) {
        self.changed_this_frame.write().unwrap().clear();
    }
    pub fn bind_buffer<T>(
        &self,
        label: Option<&str>,
        data: &mut T,
        usage: wgpu::BufferUsages,
        render_core_context: &RenderCoreContext,
    ) -> bool
    where
        T: AsBinding,
    {
        let id = data.id();
        let mut bind_data_buffer = self.buffers.write().unwrap();
        let buffer = bind_data_buffer
            .entry(id)
            .or_default();
        if data.is_dirty() || usage != buffer.usage() {
            let is_changed = buffer.bind(label, data, usage, render_core_context);
            if is_changed {
                self.changed_this_frame.write().unwrap().push(id);
            }
            return is_changed;
        }
        false
    }
}
