use nrg_core::*;
use nrg_platform::*;

use crate::config::*;

pub struct WindowSystem {
    id: SystemId,
    config: Config,
    shared_data: SharedDataRw,
    frame_count: u64,
}

impl WindowSystem {
    pub fn new(config: &Config, shared_data: &mut SharedDataRw) -> Self {
        Self {
            id: SystemId::new(),
            config: config.clone(),
            shared_data: shared_data.clone(),
            frame_count: 0,
        }
    }
}

impl System for WindowSystem {
    fn id(&self) -> SystemId {
        self.id
    }
    fn init(&mut self) {
        let pos = self.config.get_position();
        let size = self.config.get_resolution();
        let name = self.config.get_name();
        let window = Window::create(name.clone(), pos.x, pos.y, size.x, size.y);

        self.shared_data.write().unwrap().add_resource(window);
    }
    fn run(&mut self) -> bool {
        let data = self.shared_data.read().unwrap();
        let mut window_res = data.get_unique_resource_mut::<Window>();
        let result = (*window_res).update(self.frame_count);
        self.frame_count += 1;
        result
    }
    fn uninit(&mut self) {
        self.shared_data
            .write()
            .unwrap()
            .request_remove_resources_of_type::<Window>();
    }
}
