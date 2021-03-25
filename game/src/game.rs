use super::config::*;
use nrg_core::*;
use nrg_serialize::*;

use super::system::*;

const UPDATE_PHASE: &str = "UPDATE_PHASE";

#[repr(C)]
pub struct Game {
    config: Config,
    system_id: SystemId,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            config: Config::default(),
            system_id: SystemId::default(),
        }
    }
}
unsafe impl Send for Game {}
unsafe impl Sync for Game {}

impl Plugin for Game {
    fn prepare<'a>(&mut self, scheduler: &mut Scheduler, shared_data: &mut SharedDataRw) {
        let path = self.config.get_filepath();
        deserialize_from_file(&mut self.config, path);

        let mut update_phase = PhaseWithSystems::new(UPDATE_PHASE);
        let system = MySystem::new(shared_data, &self.config);

        self.system_id = system.id();

        update_phase.add_system(system);
        scheduler.create_phase(update_phase);
    }

    fn unprepare(&mut self, scheduler: &mut Scheduler) {
        let path = self.config.get_filepath();
        serialize_to_file(&self.config, path);

        let update_phase: &mut PhaseWithSystems = scheduler.get_phase_mut(UPDATE_PHASE);
        update_phase.remove_system(&self.system_id);
        scheduler.destroy_phase(UPDATE_PHASE);
    }
}
