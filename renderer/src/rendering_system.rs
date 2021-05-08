use nrg_core::*;
use nrg_graphics::*;

use nrg_resources::SharedDataRw;

pub struct RenderingSystem {
    id: SystemId,
    renderer: RendererRw,
    shared_data: SharedDataRw,
}

impl RenderingSystem {
    pub fn new(renderer: RendererRw, shared_data: &SharedDataRw) -> Self {
        Self {
            id: SystemId::new(),
            renderer,
            shared_data: shared_data.clone(),
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

    fn run(&mut self) -> bool {
        let state = self.renderer.read().unwrap().get_state();
        if state != RendererState::Prepared {
            return true;
        }

        {
            let mut renderer = self.renderer.write().unwrap();
            let data = self.shared_data.write().unwrap();
            let mut pipelines = data.get_resources_of_type_mut::<PipelineInstance>();
            renderer.draw(&mut pipelines);
        }

        true
    }
    fn uninit(&mut self) {}
}
