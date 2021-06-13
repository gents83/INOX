use nrg_core::*;
use nrg_graphics::*;

pub struct RenderingSystem {
    id: SystemId,
    renderer: RendererRw,
}

impl RenderingSystem {
    pub fn new(renderer: RendererRw) -> Self {
        Self {
            id: SystemId::new(),
            renderer,
        }
    }
}

unsafe impl Send for RenderingSystem {}
unsafe impl Sync for RenderingSystem {}

impl System for RenderingSystem {
    fn id(&self) -> SystemId {
        self.id
    }
    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {}

    fn run(&mut self) -> bool {
        let state = self.renderer.read().unwrap().state();
        if state != RendererState::Prepared {
            return true;
        }

        let mut renderer = self.renderer.write().unwrap();
        renderer.draw();

        true
    }
    fn uninit(&mut self) {}
}
