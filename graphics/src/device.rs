
pub struct Device<'a> {
    inner: super::api::backend::device::Device<'a>,
}

impl<'a> Device<'a> {
    pub fn create(instance:&'a mut super::instance::Instance) -> Device {
        let device = super::api::backend::device::Device::new(&instance.inner);
        Device {
            inner: device,
        } 
    }

    pub fn destroy(&self) {
        self.inner.delete();
    }
}