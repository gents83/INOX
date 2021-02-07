use nrg_core::*;

pub struct MySystem {
    id: SystemId,
    shared_data: SharedDataRw,
}

impl MySystem {
    pub fn new(shared_data: &SharedDataRw) -> Self {
        Self {
            id: SystemId::new(),
            shared_data: shared_data.clone(),
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
