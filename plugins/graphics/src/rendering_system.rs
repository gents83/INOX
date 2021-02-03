use nrg_core::*;

pub struct RenderingSystem {
    id: SystemId,
}

impl RenderingSystem {
    pub fn new() -> Self {
        Self {
            id: SystemId::new(),
        }
    }
} 

impl System for RenderingSystem {
    fn id(&self) -> SystemId {
        self.id
    }    
    fn init(&mut self) {
        //println!("Executing init() for RenderingSystem[{:?}]", self.id());
    }
    fn run(&mut self) -> bool {
        //println!("Executing run() for RenderingSystem[{:?}]", self.id());
        true
    }
    fn uninit(&mut self) {
        //println!("Executing uninit() for RenderingSystem[{:?}]", self.id());
    }
}
