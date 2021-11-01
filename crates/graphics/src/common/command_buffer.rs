use crate::{api::backend::BackendCommandBuffer, Device};

#[derive(Clone)]
pub struct CommandBuffer {
    inner: BackendCommandBuffer,
}

unsafe impl Send for CommandBuffer {}
unsafe impl Sync for CommandBuffer {}

impl CommandBuffer {
    pub fn new(device: &mut Device) -> Self {
        Self {
            inner: BackendCommandBuffer::create(device),
        }
    }
    pub fn execute(&self, device: &Device) {
        self.inner.execute(device);
    }
}

impl std::ops::Deref for CommandBuffer {
    type Target = BackendCommandBuffer;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for CommandBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
