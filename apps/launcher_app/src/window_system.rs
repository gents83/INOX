use nrg_core::{System, SystemId};
use nrg_platform::Window;

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
    fn should_run_when_not_focused(&self) -> bool {
        true
    }
    fn init(&mut self) {}
    fn run(&mut self) -> bool {
        self.window.update()
    }
    fn uninit(&mut self) {}
}
