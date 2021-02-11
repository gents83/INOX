use std::sync::Arc;

use nrg_core::*;
use nrg_math::*;
use nrg_platform::*;

pub struct WindowSystem {
    id: SystemId,
    shared_data: SharedDataRw,
}

impl WindowSystem {
    pub fn new(shared_data: &mut SharedDataRw) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
        }
    }
}

impl System for WindowSystem {
    fn id(&self) -> SystemId {
        self.id
    }
    fn init(&mut self) {
        //println!("Executing init() for WindowSystem[{:?}]", self.id());
        let _pos = Vector2u::new(10, 10);
        let size = Vector2u::new(1024, 768);

        let window = Window::create(
            String::from("NRG"),
            String::from("NRG - Window"),
            _pos.x,
            _pos.y,
            size.x,
            size.y,
        );
        self.shared_data.write().unwrap().add_resource(window);
    }
    fn run(&mut self) -> bool {
        //println!("Executing run() for WindowSystem[{:?}]", self.id());
        let data = self.shared_data.read().unwrap();
        let window_res = data.get_unique_resource::<Window>();
        (*window_res).update()
    }
    fn uninit(&mut self) {
        self.shared_data
            .write()
            .unwrap()
            .remove_resources_of_type::<Window>();
        //println!("Executing uninit() for WindowSystem[{:?}]", self.id());
    }
}
