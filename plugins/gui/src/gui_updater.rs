use crate::widgets::*;

use super::config::*;

use nrg_core::*;
use nrg_graphics::*;

pub struct GuiUpdater {
    id: SystemId,
    shared_data: SharedDataRw,
    config: Config,
    panel: Panel,
}

impl GuiUpdater {
    pub fn new(shared_data: &SharedDataRw, config: &Config) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
            config: config.clone(),
            panel: Panel::default(),
        }
    }
}

impl System for GuiUpdater {
    fn id(&self) -> SystemId {
        self.id
    }

    fn init(&mut self) {
        self.load_pipelines();

        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();
        self.panel.init(renderer);
    }
    fn run(&mut self) -> bool {
        true
    }
    fn uninit(&mut self) {
        self.shared_data
            .write()
            .unwrap()
            .remove_resources_of_type::<Panel>();
    }
}

impl GuiUpdater {
    fn load_pipelines(&mut self) {
        let read_data = self.shared_data.read().unwrap();
        let renderer = &mut *read_data.get_unique_resource_mut::<Renderer>();

        for pipeline_data in self.config.pipelines.iter() {
            renderer.add_pipeline(pipeline_data);
        }
    }
}
