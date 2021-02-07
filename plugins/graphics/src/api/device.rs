
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct Device {
    pub inner: super::backend::device::Device,
}

impl Device {
    pub fn create(instance:&super::instance::Instance) -> Self {
        Device {
            inner: super::backend::device::Device::new(&instance.inner) ,
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

    pub fn submit(&mut self) -> bool {
        self.inner.submit()
    }    

    pub fn recreate_swap_chain(&mut self) {
        self.inner.recreate_swap_chain();
    }
}
