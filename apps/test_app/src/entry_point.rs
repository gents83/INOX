use crate::main_system::MainSystem;
use nrg_core::{App, PhaseWithSystems, System, SystemId};
use nrg_ui::UISystem;
const UPDATE_PHASE: &str = "UPDATE_PHASE";

#[repr(C)]
pub struct EntryPoint {
    main_id: SystemId,
    ui_id: SystemId,
}

impl Default for EntryPoint {
    fn default() -> Self {
        Self {
            main_id: SystemId::default(),
            ui_id: SystemId::default(),
        }
    }
}

impl EntryPoint {
    pub fn prepare(&mut self, app: &mut App) {
        let mut update_phase = PhaseWithSystems::new(UPDATE_PHASE);

        let main_system = MainSystem::new(app.get_shared_data(), app.get_global_messenger());
        self.main_id = main_system.id();
        update_phase.add_system(main_system);

        let ui_system = UISystem::new(
            app.get_shared_data(),
            app.get_global_messenger(),
            app.get_job_handler(),
        );
        self.ui_id = ui_system.id();
        update_phase.add_system(ui_system);

        app.create_phase_before(update_phase, "RENDERING_UPDATE");
    }

    pub fn unprepare(&mut self, app: &mut App) {
        let update_phase: &mut PhaseWithSystems = app.get_phase_mut(UPDATE_PHASE);
        update_phase.remove_system(&self.main_id);
        app.destroy_phase(UPDATE_PHASE);
    }
}
