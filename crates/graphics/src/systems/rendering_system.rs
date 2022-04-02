use inox_core::{JobHandlerRw, System};

use inox_resources::SharedDataRc;
use inox_uid::generate_random_uid;

use crate::{RendererRw, RendererState};

pub const RENDERING_PHASE: &str = "RENDERING_PHASE";

pub struct RenderingSystem {
    renderer: RendererRw,
    job_handler: JobHandlerRw,
    shared_data: SharedDataRc,
}

impl RenderingSystem {
    pub fn new(
        renderer: RendererRw,
        shared_data: &SharedDataRc,
        job_handler: &JobHandlerRw,
    ) -> Self {
        Self {
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
        let renderer = self.renderer.clone();

        self.job_handler.write().unwrap().add_job(
            &generate_random_uid(),
            "Render Draw",
            move || {
                {
                    let mut renderer = renderer.write().unwrap();
                    renderer.change_state(RendererState::Drawing);
                }

                {
                    let renderer = renderer.read().unwrap();
                    renderer.draw();
                }

                {
                    let mut renderer = renderer.write().unwrap();
                    renderer.change_state(RendererState::Submitted);
                }
            },
        );
        true
    }
    fn uninit(&mut self) {}
}
