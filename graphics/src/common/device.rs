use crate::api::backend::BackendDevice;

#[derive(Clone)]
pub struct Device {
    inner: BackendDevice,
}

impl std::ops::Deref for Device {
    type Target = BackendDevice;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for Device {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Device {
    pub fn create(instance: &super::instance::Instance, enable_validation: bool) -> Self {
        Device {
            inner: BackendDevice::new(&*instance, enable_validation),
        }
    }

    pub fn destroy(&self) {
        self.inner.delete();
    }

    pub fn begin_frame(&mut self) -> bool {
        self.inner.begin_frame()
    }

    pub fn end_frame(&self) {
        self.inner.end_frame();
    }

    pub fn submit(&self) {
        self.inner.submit();
    }

    pub fn present(&mut self) -> bool {
        self.inner.present()
    }

    pub fn recreate_swap_chain(&mut self) {
        self.inner.recreate_swap_chain();
    }
}
