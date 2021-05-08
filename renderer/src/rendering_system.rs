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
    fn init(&mut self) {}

    fn run(&mut self) -> (bool, Vec<Job>) {
        let state = self.renderer.read().unwrap().get_state();
        if state != RendererState::Prepared {
            return (true, Vec::new());
        }

        {
            let mut renderer = self.renderer.write().unwrap();
            renderer.draw();
        }

        (true, Vec::new())
    }
    fn uninit(&mut self) {}
}
