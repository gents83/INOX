use sabi_blender::LogicNodeRegistry;
use sabi_core::System;
use sabi_messenger::MessengerRw;
use sabi_resources::{ConfigBase, SharedDataRc};
use sabi_serialize::SerializeFile;

use crate::{config::Config, logic_nodes::MoveNode};

pub struct ExampleGame {
    _global_messenger: MessengerRw,
    _shared_data: SharedDataRc,
}

impl ExampleGame {
    pub fn new(global_messenger: &MessengerRw, shared_data: &SharedDataRc) -> Self {
        Self::register_nodes(shared_data);

        Self {
            _global_messenger: global_messenger.clone(),
            _shared_data: shared_data.clone(),
        }
    }

    fn register_nodes(shared_data: &SharedDataRc) {
        if let Some(registry) = shared_data.get_singleton_mut::<LogicNodeRegistry>() {
            registry.register_node::<MoveNode>();
        }
    }
}

impl System for ExampleGame {
    fn read_config(&mut self, plugin_name: &str) {
        let mut config = Config::default();
        config.load_from_file(config.get_filepath(plugin_name).as_path());
    }
    fn should_run_when_not_focused(&self) -> bool {
        false
    }

    fn init(&mut self) {}

    fn run(&mut self) -> bool {
        true
    }
    fn uninit(&mut self) {}
}
