use std::sync::Arc;

use nrg_core::*;
use nrg_math::*;
use nrg_platform::*;


pub struct WindowSystem {
    id: SystemId,
    shared_data: Arc<SharedData>,
}

impl WindowSystem {
    pub fn new(game_name: String, shared_data: &mut Arc<SharedData>) -> Self {
        let _pos = Vector2u::new(10, 10);
        let size = Vector2u::new(1024, 768);
    
        let window =  Window::create( game_name.clone(),
            game_name.clone(),
            _pos.x, _pos.y,
            size.x, size.y );

        let data = Arc::get_mut(shared_data).unwrap();
        data.add_resource(window);

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
    }
    fn run(&mut self) -> bool {
        //println!("Executing run() for WindowSystem[{:?}]", self.id());
        let window_res = self.shared_data.get_unique_resource::<Window>();
        (*window_res).update()
    }
    fn uninit(&mut self) {
        //println!("Executing uninit() for WindowSystem[{:?}]", self.id());
    }
}
