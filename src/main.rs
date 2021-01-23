use nrg_app::*;

const UPDATE_PHASE:&str = "UPDATE_PHASE";

struct MySystem {
    id: SystemId,
}

impl MySystem {
    pub fn new() -> Self {
        Self {
            id: SystemId::new(),
        }
    }
} 

impl System for MySystem {
    type In = ();
    type Out = ();

    fn id(&self) -> SystemId {
        self.id
    }    
    fn init(&mut self) {
        println!("Executing init() for MySystem[{:?}]", self.id());
    }
    fn run(&mut self, _input: Self::In) -> Self::Out {
        println!("Executing run() for MySystem[{:?}]", self.id());
    }
    fn uninit(&mut self) {
        println!("Executing uninit() for MySystem[{:?}]", self.id());
    }
}

fn main() {
    let mut app = App::new();

    let mut update_phase = PhaseWithSystems::new(UPDATE_PHASE); 
    let my_system = MySystem::new();
    
    update_phase.add_system(my_system);
    app.create_phase(update_phase); 

    
    app.run_once();
}