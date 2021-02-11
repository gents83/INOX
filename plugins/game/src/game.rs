use nrg_core::*;
use std::any::type_name;

use super::system::*;

const UPDATE_PHASE: &str = "UPDATE_PHASE";

#[repr(C)]
pub struct Game {
    pub game_name: String,
    system_id: SystemId,
}

impl Default for Game {
    fn default() -> Self {
        let game = Self {
            game_name: String::from("NRG Game"),
            system_id: SystemId::default(),
        };
        println!("Created {} plugin", type_name::<Self>().to_string());
        game
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        println!("Destroyed {} plugin", type_name::<Self>().to_string());
    }
}

unsafe impl Send for Game {}
unsafe impl Sync for Game {}

impl Plugin for Game {
    fn prepare<'a>(&mut self, scheduler: &mut Scheduler, shared_data: &mut SharedDataRw) {
        println!("Prepare {} plugin", type_name::<Self>().to_string());
        let mut update_phase = PhaseWithSystems::new(UPDATE_PHASE);
        let system = MySystem::new(shared_data);

        self.system_id = system.id();

        update_phase.add_system(system);
        scheduler.create_phase(update_phase);
    }

    fn unprepare(&mut self, scheduler: &mut Scheduler) {
        let update_phase: &mut PhaseWithSystems = scheduler.get_phase_mut(UPDATE_PHASE);
        update_phase.remove_system(&self.system_id);
        scheduler.destroy_phase(UPDATE_PHASE);
        println!("Unprepare {} plugin", type_name::<Self>().to_string());
    }
}
