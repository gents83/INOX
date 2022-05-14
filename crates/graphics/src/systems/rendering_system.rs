use inox_core::{
    implement_unique_system_uid, ContextRc, JobHandlerRw, JobHandlerTrait, JobPriority, System,
    INDEPENDENT_JOB_ID,
};

use crate::{RendererRw, RendererState};

pub const RENDERING_PHASE: &str = "RENDERING_PHASE";

pub struct RenderingSystem {
    renderer: RendererRw,
    job_handler: JobHandlerRw,
}

impl RenderingSystem {
    pub fn new(renderer: RendererRw, context: &ContextRc) -> Self {
        Self {
            renderer,
            job_handler: context.job_handler().clone(),
        }
    }
}

unsafe impl Send for RenderingSystem {}
unsafe impl Sync for RenderingSystem {}

implement_unique_system_uid!(RenderingSystem);

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
        }
        {
            let renderer = self.renderer.read().unwrap();
            renderer.draw()
        };

        {
            let mut renderer = self.renderer.write().unwrap();
            renderer.change_state(RendererState::Submitted);
        }

        {
            let renderer = self.renderer.clone();
            self.job_handler.add_job(
                &INDEPENDENT_JOB_ID,
                "Render Draw",
                JobPriority::High,
                move || {
                    let renderer = renderer.read().unwrap();
                    renderer.present();
                },
            );
        }
        true
    }
    fn uninit(&mut self) {}
}
