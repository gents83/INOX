use crate::main_system::MainSystem;
use nrg_core::{App, PhaseWithSystems, System, SystemId};

const UPDATE_PHASE: &str = "UPDATE_PHASE";

#[repr(C)]
pub struct EntryPoint {
    id: SystemId,
}

impl Default for EntryPoint {
    fn default() -> Self {
        Self {
            id: SystemId::new(),
        }
    }
}

impl EntryPoint {
    pub fn prepare(&mut self, app: &mut App) {
        let mut update_phase = PhaseWithSystems::new(UPDATE_PHASE);
        let system = MainSystem::new(
            app.get_shared_data(),
            app.get_global_messenger(),
            app.get_job_handler(),
        );
        self.id = system.id();
        update_phase.add_system(system);
        app.create_phase_before(update_phase, "RENDERING_UPDATE");
    }

    pub fn unprepare(&mut self, app: &mut App) {
        let update_phase: &mut PhaseWithSystems = app.get_phase_mut(UPDATE_PHASE);
        update_phase.remove_system(&self.id);
        app.destroy_phase(UPDATE_PHASE);
    }
}
