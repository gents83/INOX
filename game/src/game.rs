use nrg_app::*;

use super::system::*;

const UPDATE_PHASE:&str = "UPDATE_PHASE";

#[repr(C)]
pub struct Game {
    pub game_name: String,
    system_id: SystemId,
}

impl Default for Game {
    fn default() -> Self {
        Self{
            game_name: String::from("Game"),
            system_id: SystemId::default(),
        }
    }
}

impl Plugin for Game {
    fn prepare(&mut self, scheduler: &mut Scheduler, _shared_data: &mut SharedData) {    
        println!("Prepare {} plugin", self.game_name);
        
        let mut update_phase = PhaseWithSystems::new(UPDATE_PHASE);
        let system = MySystem::new();

        self.system_id = system.id();

        update_phase.add_system(system);
        scheduler.create_phase(update_phase); 
    }
    
    fn unprepare(&mut self, scheduler: &mut Scheduler, _shared_data: &mut SharedData) {   
        let update_phase:&mut PhaseWithSystems = scheduler.get_phase_mut(UPDATE_PHASE);
        update_phase.remove_system(&self.system_id);
        scheduler.destroy_phase(UPDATE_PHASE);
        println!("Unprepare {} plugin", self.game_name);
    }
}