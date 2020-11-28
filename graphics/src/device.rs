pub struct Device<'a>(super::api::backend::device::Device<'a>);

impl<'a> Device<'a> {
    pub fn create(instance:&'a mut super::instance::Instance) -> Device {
        let device = super::api::backend::device::Device::new(&mut instance.inner);
        Device(device) 
    }
}