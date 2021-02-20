use std::collections::HashMap;

use crate::widgets::*;

use super::config::*;

use nrg_core::*;
use nrg_platform::*;

pub struct GuiUpdater {
    id: SystemId,
    shared_data: SharedDataRw,
    config: Config,
    keys: HashMap<Key, InputState>,
    mouse: MouseEvent,
}

impl GuiUpdater {
    pub fn new(shared_data: &SharedDataRw, config: &Config) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
            config: config.clone(),
            keys: HashMap::new(),
            mouse: MouseEvent::default(),
        }
    }
}

impl System for GuiUpdater {
    fn id(&self) -> SystemId {
        self.id
    }
    fn init(&mut self) {
        let panel = Panel::default();
        self.shared_data.write().unwrap().add_resource(panel);
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
