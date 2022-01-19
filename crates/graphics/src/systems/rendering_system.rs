use sabi_core::{JobHandlerRw, System};
use sabi_messenger::MessengerRw;
use sabi_resources::{DataTypeResource, Resource, SharedDataRc};
use sabi_serialize::generate_random_uid;

use crate::{RendererRw, RendererState, View};

pub const RENDERING_PHASE: &str = "RENDERING_PHASE";

pub struct RenderingSystem {
    view: Resource<View>,
    renderer: RendererRw,
    job_handler: JobHandlerRw,
    shared_data: SharedDataRc,
}

impl RenderingSystem {
    pub fn new(
        renderer: RendererRw,
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        job_handler: &JobHandlerRw,
    ) -> Self {
        Self {
            view: View::new_resource(shared_data, global_messenger, generate_random_uid(), 0),
            renderer,
            job_handler: job_handler.clone(),
            shared_data: shared_data.clone(),
        }
    }
}

unsafe impl Send for RenderingSystem {}
unsafe impl Sync for RenderingSystem {}

impl System for RenderingSystem {
    fn read_config(&mut self, _plugin_name: &str) {}
    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {}

    fn run(&mut self) -> bool {
        let state = self.renderer.read().unwrap().state();
        if state != RendererState::Prepared {
            return true;
        }

        {
            let mut renderer = self.renderer.write().unwrap();
            renderer.change_state(RendererState::Drawing);
            renderer.update_shader_data(self.view.get().view(), self.view.get().proj());
        }

        {
            let renderer = self.renderer.read().unwrap();
            renderer.draw();
        }

        {
            let mut renderer = self.renderer.write().unwrap();
            renderer.change_state(RendererState::Submitted);
        }
        true
    }
    fn uninit(&mut self) {}
}
