use crate::handle::*;

pub struct Window {
    pub handle: Handle,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub name: String,
    pub title: String,
}

impl Window {
    pub fn create(
        _name: String,
        _title: String,
        _x: u32,
        _y: u32,
        _width: u32,
        _height: u32) -> Window {

        Window::new(_name, _title, _x, _y, _width, _height)

    }

    pub fn update(&self) -> bool {
        let is_ended = Window::internal_update(self);
        is_ended
    }
}