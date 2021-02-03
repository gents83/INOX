
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct Device {
    pub inner: Rc<RefCell<super::backend::device::Device>>,
}

impl Device {
    pub fn create(instance:&super::instance::Instance) -> Self {
        let device = Rc::new( RefCell::new( super::backend::device::Device::new(&instance.inner) ) );
        Device {
            inner: device,
        } 
    }

    pub fn destroy(&self) {
        self.inner.borrow().delete();
    }

    pub fn begin_frame(&self) -> bool {
        self.inner.borrow_mut().begin_frame()
    }

    pub fn end_frame(&self) {
        self.inner.borrow().end_frame();
    }

    pub fn submit(&self) -> bool {
        self.inner.borrow_mut().submit()
    }    

    pub fn recreate_swap_chain(&self) {
        self.inner.borrow_mut().recreate_swap_chain();
    }
}
