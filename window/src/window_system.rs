use nrg_core::*;
use nrg_platform::*;

pub struct WindowSystem {
    id: SystemId,
    window: Window,
}

impl WindowSystem {
    pub fn new(window: Window) -> Self {
        Self {
            id: SystemId::new(),
            window,
        }
    }
}

impl System for WindowSystem {
    fn id(&self) -> SystemId {
        self.id
    }
    fn init(&mut self) {}
    fn run(&mut self) -> bool {
        self.window.update()
    }
    fn uninit(&mut self) {}
}
