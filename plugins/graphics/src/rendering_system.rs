use std::sync::Arc;

use nrg_core::*;
use nrg_math::*;
use nrg_platform::*;
use crate::api::renderer::*;

pub struct RenderingSystem {
    id: SystemId,
    shared_data: SharedDataRw,
}

impl RenderingSystem {
    pub fn new(shared_data: &SharedDataRw) -> Self {
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
    
        let renderer = {
            let read_data = self.shared_data.read().unwrap();
            let res_window = read_data.get_unique_resource::<Window>();
            let window = &*res_window;
            
            let mut renderer = Renderer::new(window.get_handle(), false);
            let size = Vector2u::new(window.get_width(), window.get_heigth());
            renderer.set_viewport_size(size);
            renderer
        };
        self.shared_data.write().unwrap().add_resource(renderer);
    }
    fn run(&mut self) -> bool {
        //println!("Executing run() for RenderingSystem[{:?}]", self.id());
        true
    }
    fn uninit(&mut self) {
        //println!("Executing uninit() for RenderingSystem[{:?}]", self.id());
    }
}
