use nrg_platform::Handle;

#[derive(Clone)]
pub struct Instance {
    pub inner: crate::api::backend::instance::Instance,
}

impl Instance {
    pub fn create(handle: &Handle, debug_enabled: bool) -> Instance {
        Self {
            inner: crate::api::backend::instance::Instance::new(handle, debug_enabled),
        }
    }

    pub fn destroy(&self) {
        self.inner.delete();
    }
}