use nrg_core::{define_plugin, App, PhaseWithSystems, Plugin, System, SystemId};
use nrg_ui::UISystem;

use crate::content_browser_updater::ContentBrowserUpdater;

const CONTENT_BROWSER_UPDATE_PHASE: &str = "CONTENT_BROWSER_UPDATE_PHASE";

#[repr(C)]
pub struct ContentBrowser {
    updater_id: SystemId,
    ui_id: SystemId,
    renderer_id: SystemId,
}
define_plugin!(ContentBrowser);

impl Default for ContentBrowser {
    fn default() -> Self {
        Self {
            updater_id: SystemId::default(),
            ui_id: SystemId::default(),
            renderer_id: SystemId::default(),
        }
    }
}

impl Plugin for ContentBrowser {
    fn name(&self) -> &str {
        "nrg_content_browser"
    }
    fn prepare(&mut self, app: &mut App) {
        let mut update_phase = PhaseWithSystems::new(CONTENT_BROWSER_UPDATE_PHASE);
        let system = ContentBrowserUpdater::new(app.get_shared_data(), app.get_global_messenger());
        self.updater_id = ContentBrowserUpdater::id();
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
        let update_phase: &mut PhaseWithSystems = app.get_phase_mut(CONTENT_BROWSER_UPDATE_PHASE);
        update_phase.remove_system(&self.ui_id);
        update_phase.remove_system(&self.updater_id);
        app.destroy_phase(CONTENT_BROWSER_UPDATE_PHASE);
    }
}
