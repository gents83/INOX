use crate::events::*;
use crate::handle::*;
use crate::input::*;
pub struct Window {
    handle: Handle,
    width: u32,
    height: u32,
    events: Events,
}

unsafe impl Send for Window {}
unsafe impl Sync for Window {}

impl Window {
    pub fn create(title: String, x: u32, y: u32, width: u32, height: u32) -> Self {
        let mut events = Events::default();
        let handle = Window::create_handle(title, x, y, width, height);

        events.register_event::<KeyEvent>();

        Self {
            handle,
            width,
            height,
            events,
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

    pub fn get_events(&self) -> &Events {
        &self.events
    }

    pub fn update(&mut self) -> bool {
        self.events.clear_events::<KeyEvent>();
        Window::internal_update(&mut self.events)
    }
}
