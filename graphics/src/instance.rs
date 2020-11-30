use nrg_platform::Handle;

pub struct Instance {
    pub inner: super::api::backend::instance::Instance,
}

impl Instance {
    pub fn create(handle: &Handle) -> Instance {
        Instance {
            inner: super::api::backend::instance::Instance::new(handle, false),
        } 
    }

    pub fn destroy(&self) {
        self.inner.delete();
    }
}