pub struct Device<'a> {
    instance: &'a mut super::Instance,
}

impl<'a> Device<'a> {
    pub fn new(mut instance: &'a mut super::Instance) -> Device {
        Self {
            instance: instance,
        }
    }
}