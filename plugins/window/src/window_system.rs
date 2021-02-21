use nrg_core::*;
use nrg_platform::*;

use crate::config::*;

pub struct WindowSystem {
    id: SystemId,
    shared_data: SharedDataRw,
}

impl WindowSystem {
    pub fn new(config: &Config, shared_data: &mut SharedDataRw) -> Self {
        let pos = config.get_position();
        let size = config.get_resolution();
        let name = config.get_name();
        let window = Window::create(name.clone(), pos.x, pos.y, size.x, size.y);

        shared_data.write().unwrap().add_resource(window);

        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
        }
    }
}

impl Drop for WindowSystem {
    fn drop(&mut self) {
        self.shared_data
            .write()
            .unwrap()
            .request_remove_resources_of_type::<Window>();
    }
}

impl System for WindowSystem {
    fn id(&self) -> SystemId {
        self.id
    }
    fn init(&mut self) {}
    fn run(&mut self) -> bool {
        let data = self.shared_data.read().unwrap();
        let mut window_res = data.get_unique_resource_mut::<Window>();
        (*window_res).update()
    }
    fn uninit(&mut self) {}
}
