use nrg_platform::Handle;

pub struct Instance {
    pub inner: super::backend::instance::Instance,
}

impl Instance {
    pub fn create(handle: &Handle, debug_enabled: bool) -> Instance {
        Self {
            inner: super::backend::instance::Instance::new(handle, debug_enabled),
        }
    }

    pub fn destroy(&self) {
        self.inner.delete();
    }
}