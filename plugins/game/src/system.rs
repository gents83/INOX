use nrg_core::*;


pub struct MySystem {
    id: SystemId,
}

impl MySystem {
    pub fn new(game_name: String) -> Self {
        Self {
            id: SystemId::new(),
        }
    }
} 

impl System for MySystem {
    fn id(&self) -> SystemId {
        self.id
    }    
    fn init(&mut self) {
        //println!("Executing init() for MySystem[{:?}]", self.id());
    }
    fn run(&mut self) -> bool {
        //println!("Executing run() for MySystem[{:?}]", self.id());
        true
    }
    fn uninit(&mut self) {
        //println!("Executing uninit() for MySystem[{:?}]", self.id());
    }
}
