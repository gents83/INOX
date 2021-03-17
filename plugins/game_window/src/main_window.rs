use nrg_core::*;
use nrg_serialize::*;
use std::any::type_name;

use crate::config::*;
use crate::window_system::*;

const MAIN_WINDOW_PHASE: &str = "MAIN_WINDOW_PHASE";

#[repr(C)]
pub struct MainWindow {
    config: Config,
    system_id: SystemId,
}

impl Default for MainWindow {
    fn default() -> Self {
        println!("Created {} plugin", type_name::<Self>().to_string());
        Self {
            config: Config::default(),
            system_id: SystemId::default(),
        }
    }
}

impl Drop for MainWindow {
    fn drop(&mut self) {
        println!("Destroyed {} plugin", type_name::<Self>().to_string());
    }
}

unsafe impl Send for MainWindow {}
unsafe impl Sync for MainWindow {}

impl Plugin for MainWindow {
    fn prepare(&mut self, scheduler: &mut Scheduler, shared_data: &mut SharedDataRw) {
        let path = self.config.get_filepath();
        deserialize(&mut self.config, path);

        let mut update_phase = PhaseWithSystems::new(MAIN_WINDOW_PHASE);
        let system = WindowSystem::new(&self.config, shared_data);

        self.system_id = system.id();

        update_phase.add_system(system);
        scheduler.create_phase(update_phase);
    }

    fn unprepare(&mut self, scheduler: &mut Scheduler) {
        let path = self.config.get_filepath();
        serialize(&self.config, path);

        let update_phase: &mut PhaseWithSystems = scheduler.get_phase_mut(MAIN_WINDOW_PHASE);
        update_phase.remove_system(&self.system_id);
        scheduler.destroy_phase(MAIN_WINDOW_PHASE);
    }
}
