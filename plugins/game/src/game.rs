use nrg_core::*;

use super::system::*;

const UPDATE_PHASE:&str = "UPDATE_PHASE";

#[repr(C)]
pub struct Game {
    pub game_name: String,
    system_id: SystemId,
}

impl Default for Game {
    fn default() -> Self {
        let game = Self{
            game_name: String::from("NRG Game"),
            system_id: SystemId::default(),
        };
        println!("Created {} plugin", game.game_name);
        game
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        println!("Destroyed {} plugin", self.game_name);
    }
}

unsafe impl Send for Game {}
unsafe impl Sync for Game {}

impl Plugin for Game {
    fn prepare(&mut self, scheduler: &mut Scheduler, _shared_data: &mut SharedData) {    
        println!("Prepare {} plugin", self.game_name);
        
        let mut update_phase = PhaseWithSystems::new(UPDATE_PHASE);
        let system = MySystem::new(self.game_name.clone());

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