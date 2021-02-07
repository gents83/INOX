use std::sync::Arc;

use nrg_core::*;
use nrg_platform::*;
use crate::api::renderer::*;

pub struct RenderingSystem {
    id: SystemId,
    shared_data: Arc<SharedData>,
}

impl RenderingSystem {
    pub fn new(shared_data: &Arc<SharedData>) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
        }
    }
} 

impl System for RenderingSystem {
    fn id(&self) -> SystemId {
        self.id
    }    
    fn init(&mut self) {
        //println!("Executing init() for RenderingSystem[{:?}]", self.id());
        let window_res = self.shared_data.get_unique_resource::<Window>();
        let mut renderer = Renderer::new(&((*window_res).handle), false);
    }
    fn run(&mut self) -> bool {
        //println!("Executing run() for RenderingSystem[{:?}]", self.id());
        true
    }
    fn uninit(&mut self) {
        //println!("Executing uninit() for RenderingSystem[{:?}]", self.id());
    }
}
