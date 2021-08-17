use nrg_core::{System, SystemId};
use nrg_resources::{DataTypeResource, ResourceRef, SharedData, SharedDataRw};

use crate::{RendererRw, RendererState, ViewInstance, ViewRc};

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
            view: ResourceRef::default(),
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

        let width = self.view.resource().get().width();
        let height = self.view.resource().get().height();
        let view = self.view.resource().get().view();
        let proj = self.view.resource().get().proj();

        let mut renderer = self.renderer.write().unwrap();
        renderer.draw(width as _, height as _, &view, &proj);

        true
    }
    fn uninit(&mut self) {}
}
