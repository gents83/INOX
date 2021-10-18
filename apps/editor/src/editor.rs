use super::config::*;
use nrg_core::*;

use nrg_resources::ConfigBase;
use nrg_serialize::*;
use nrg_ui::UISystem;

use super::editor_updater::*;

const EDITOR_UPDATE_PHASE: &str = "EDITOR_UPDATE_PHASE";

#[repr(C)]
pub struct Editor {
    config: Config,
    updater_id: SystemId,
    ui_id: SystemId,
    renderer_id: SystemId,
}
define_plugin!(Editor);

impl Default for Editor {
    fn default() -> Self {
        Self {
            config: Config::default(),
            updater_id: SystemId::default(),
            ui_id: SystemId::default(),
            renderer_id: SystemId::default(),
        }
    }
}

impl Plugin for Editor {
    fn prepare(&mut self, app: &mut App) {
        self.config = read_from_file(self.config.get_filepath().as_path());

        let mut update_phase = PhaseWithSystems::new(EDITOR_UPDATE_PHASE);
        let system = EditorUpdater::new(
            app.get_shared_data(),
            app.get_global_messenger(),
            &self.config,
        );
        self.updater_id = system.id();
        update_phase.add_system(system);

        let ui_system = UISystem::new(
            app.get_shared_data(),
            app.get_global_messenger(),
            app.get_job_handler(),
        );
        self.ui_id = ui_system.id();
        update_phase.add_system(ui_system);

        app.create_phase_before(update_phase, "RENDERING_UPDATE");
    }

    fn unprepare(&mut self, app: &mut App) {
        serialize_to_file(&self.config, self.config.get_filepath().as_path());

        let update_phase: &mut PhaseWithSystems = app.get_phase_mut(EDITOR_UPDATE_PHASE);
        update_phase.remove_system(&self.updater_id);
        app.destroy_phase(EDITOR_UPDATE_PHASE);
    }
}
