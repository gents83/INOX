use nrg_core::*;
use nrg_graphics::*;
use nrg_resources::{DataResource, Resource, ResourceBase, SharedData, SharedDataRw};

pub struct RenderingSystem {
    id: SystemId,
    view_index: usize,
    view: ViewRc,
    renderer: RendererRw,
    shared_data: SharedDataRw,
}

impl RenderingSystem {
    pub fn new(renderer: RendererRw, shared_data: &SharedDataRw) -> Self {
        Self {
            id: SystemId::new(),
            view_index: 0,
            view: Resource::default::<ViewInstance>(),
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
    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {
        if !SharedData::has_resources_of_type::<ViewInstance>(&self.shared_data) {
            self.view = ViewInstance::create_from_data(&self.shared_data, self.view_index as _);
        }
    }

    fn run(&mut self) -> bool {
        let state = self.renderer.read().unwrap().state();
        if state != RendererState::Prepared {
            return true;
        }

        let (view, proj) = {
            let view = *self.view.get::<ViewInstance>().view();
            let proj = *self.view.get::<ViewInstance>().proj();
            (view, proj)
        };

        let mut renderer = self.renderer.write().unwrap();
        renderer.draw(&view, &proj);

        true
    }
    fn uninit(&mut self) {}
}
