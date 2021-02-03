use nrg_core::*;
use nrg_math::*;
use nrg_platform::*;


pub struct WindowSystem {
    id: SystemId,
    window: Box<Window>,
}

impl WindowSystem {
    pub fn new(game_name: String) -> Self {
        let _pos = Vector2u::new(10, 10);
        let size = Vector2u::new(1024, 768);
    
        let window =  Box::new(Window::create( game_name.clone(),
            game_name.clone(),
            _pos.x, _pos.y,
            size.x, size.y ));

        Self {
            id: SystemId::new(),
            window,
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
        self.window.update()
    }
    fn uninit(&mut self) {
        //println!("Executing uninit() for WindowSystem[{:?}]", self.id());
    }
}
