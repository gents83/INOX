
pub struct Device {
    pub inner: super::api::backend::device::Device,
}

impl Device {
    pub fn create(instance:&super::instance::Instance) -> Self {
        let device = super::api::backend::device::Device::new(&instance.inner);
        Device {
            inner: device,
        } 
    }

    pub fn get_internal_device(&self) -> &super::api::backend::device::Device {
        &self.inner
    }

    pub fn destroy(&mut self) {
        self.inner.delete();
    }

    pub fn begin_frame(&mut self) {
        self.inner.begin_frame();
    }

    pub fn end_frame(&self) {
        self.inner.end_frame();
    }

    pub fn submit(&mut self) {
        self.inner.submit();
    }
}
