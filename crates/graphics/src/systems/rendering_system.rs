use inox_core::{implement_unique_system_uid, ContextRc, System};

use crate::{RendererRw, RendererState};

pub const RENDERING_PHASE: &str = "RENDERING_PHASE";

pub struct RenderingSystem {
    renderer: RendererRw,
}

impl RenderingSystem {
    pub fn new(renderer: RendererRw, _context: &ContextRc) -> Self {
        Self { renderer }
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
            renderer.update_passes();

            renderer.present();

            renderer.change_state(RendererState::Submitted);
        };
        true
    }
    fn uninit(&mut self) {}
}
