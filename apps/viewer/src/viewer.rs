use nrg_core::{define_plugin, App, PhaseWithSystems, Plugin, System, SystemId};

use nrg_ui::UISystem;

use crate::systems::viewer_system::ViewerSystem;

const VIEWER_UPDATE_PHASE: &str = "VIEWER_UPDATE_PHASE";

#[repr(C)]
pub struct Viewer {
    updater_id: SystemId,
    renderer_id: SystemId,
    ui_id: SystemId,
}
define_plugin!(Viewer);

impl Default for Viewer {
    fn default() -> Self {
        Self {
            updater_id: SystemId::default(),
            renderer_id: SystemId::default(),
            ui_id: SystemId::default(),
        }
    }
}

impl Plugin for Viewer {
    fn name(&self) -> &str {
        "nrg_viewer"
    }
    fn prepare(&mut self, app: &mut App) {
        let mut update_phase = PhaseWithSystems::new(VIEWER_UPDATE_PHASE);
        let system = ViewerSystem::new(app.get_shared_data(), app.get_global_messenger());
        self.updater_id = ViewerSystem::id();
        update_phase.add_system(system);

        let mut ui_system = UISystem::new(
            app.get_shared_data(),
            app.get_global_messenger(),
            app.get_job_handler(),
        );
        ui_system.read_config(self.name());
        self.ui_id = UISystem::id();
        update_phase.add_system(ui_system);

        app.create_phase_before(update_phase, "RENDERING_UPDATE");
    }

    fn unprepare(&mut self, app: &mut App) {
        let update_phase: &mut PhaseWithSystems = app.get_phase_mut(VIEWER_UPDATE_PHASE);
        update_phase.remove_system(&self.ui_id);
        update_phase.remove_system(&self.updater_id);
        app.destroy_phase(VIEWER_UPDATE_PHASE);
    }
}
