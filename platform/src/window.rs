use crate::handle::*;

pub struct Window {
    handle: Handle,
    width: u32,
    height: u32,
}

unsafe impl Send for Window {}
unsafe impl Sync for Window {}


impl Window {
    pub fn create(name: String, title: String, x: u32, y: u32, width: u32, height: u32) -> Self {
        let handle = Window::new(name.clone(), title.clone(), x, y, width, height);        
        Self {
            handle,
            width,
            height,
        }      
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_heigth(&self) -> u32 {
        self.height
    }

    pub fn get_handle(&self) -> &Handle {
        &self.handle
    }

    pub fn update(&self) -> bool {
        Window::internal_update(self)
    }
}
