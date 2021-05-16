use super::config::*;
use nrg_core::*;
use nrg_resources::ConfigBase;
use nrg_serialize::*;

use super::editor_updater::*;

const EDITOR_UPDATE_PHASE: &str = "EDITOR_UPDATE_PHASE";

#[repr(C)]
pub struct Editor {
    config: Config,
    updater_id: SystemId,
    renderer_id: SystemId,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            config: Config::default(),
            updater_id: SystemId::default(),
            renderer_id: SystemId::default(),
        }
    }
}

unsafe impl Send for Editor {}
unsafe impl Sync for Editor {}

impl Plugin for Editor {
    fn prepare(&mut self, app: &mut App) {
        let path = self.config.get_filepath();
        deserialize_from_file(&mut self.config, path);

        let mut update_phase = PhaseWithSystems::new(EDITOR_UPDATE_PHASE);
        let system = EditorUpdater::new(
            app.get_shared_data(),
            app.get_global_messenger(),
            &self.config,
        );
        self.updater_id = system.id();
        update_phase.add_system(system);
        app.create_phase_before(update_phase, "RENDERING_UPDATE");
    }

    fn unprepare(&mut self, app: &mut App) {
        let path = self.config.get_filepath();
        serialize_to_file(&self.config, path);

        let update_phase: &mut PhaseWithSystems = app.get_phase_mut(EDITOR_UPDATE_PHASE);
        update_phase.remove_system(&self.updater_id);
        app.destroy_phase(EDITOR_UPDATE_PHASE);
    }
}
