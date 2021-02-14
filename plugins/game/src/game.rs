use super::config::*;
use nrg_core::*;
use nrg_serialize::*;
use std::any::type_name;

use super::system::*;

const GAME_CFG_NAME: &str = "game.cfg";
const UPDATE_PHASE: &str = "UPDATE_PHASE";

#[repr(C)]
pub struct Game {
    config: Config,
    system_id: SystemId,
}

impl Default for Game {
    fn default() -> Self {
        println!("Created {} plugin", type_name::<Self>().to_string());
        Self {
            config: Config::default(),
            system_id: SystemId::default(),
        }
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
        let path = self.config.get_folder().join(GAME_CFG_NAME);
        deserialize(&mut self.config, path);

        let mut update_phase = PhaseWithSystems::new(UPDATE_PHASE);
        let system = MySystem::new(shared_data, &self.config);

        self.system_id = system.id();

        update_phase.add_system(system);
        scheduler.create_phase(update_phase);
    }

    fn unprepare(&mut self, scheduler: &mut Scheduler) {
        let path = self.config.get_folder().join(GAME_CFG_NAME);
        serialize(&self.config, path);

        let update_phase: &mut PhaseWithSystems = scheduler.get_phase_mut(UPDATE_PHASE);
        update_phase.remove_system(&self.system_id);
        scheduler.destroy_phase(UPDATE_PHASE);
    }
}
