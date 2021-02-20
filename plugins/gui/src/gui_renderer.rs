use crate::widgets::*;

use super::config::*;

use nrg_core::*;
use nrg_graphics::*;

pub struct GuiRenderer {
    id: SystemId,
    shared_data: SharedDataRw,
    config: Config,
}

impl GuiRenderer {
    pub fn new(shared_data: &SharedDataRw, config: &Config) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
            config: config.clone(),
        }
    }
}

impl System for GuiRenderer {
    fn id(&self) -> SystemId {
        self.id
    }
    fn init(&mut self) {
        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();
        for pipeline_data in self.config.pipelines.iter() {
            renderer.add_pipeline(pipeline_data);
        }
    }
    fn run(&mut self) -> bool {
        let read_data = self.shared_data.read().unwrap();

        let panel = &mut *read_data.get_unique_resource_mut::<Panel>();
        panel.draw();

        true
    }
    fn uninit(&mut self) {}
}
