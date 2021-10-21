use nrg_core::{
    define_plugin, App, PhaseWithSystems, Plugin, PluginHolder, PluginId, System, SystemId,
};
use nrg_resources::ConfigBase;
use nrg_serialize::{read_from_file, serialize_to_file};

use crate::{config::Config, systems::viewer_system::ViewerSystem};

const VIEWER_UPDATE_PHASE: &str = "VIEWER_UPDATE_PHASE";

#[repr(C)]
pub struct Viewer {
    config: Config,
    updater_id: SystemId,
    renderer_id: SystemId,
}
define_plugin!(Viewer);

impl Default for Viewer {
    fn default() -> Self {
        Self {
            config: Config::default(),
            updater_id: SystemId::default(),
            renderer_id: SystemId::default(),
        }
    }
}

impl Plugin for Viewer {
    fn prepare(&mut self, app: &mut App) {
        self.config = read_from_file(self.config.get_filepath().as_path());

        let mut update_phase = PhaseWithSystems::new(VIEWER_UPDATE_PHASE);
        let system = ViewerSystem::new(
            app.get_shared_data(),
            app.get_global_messenger(),
            &self.config,
        );
        self.updater_id = system.id();
        update_phase.add_system(system);

        app.create_phase_before(update_phase, "RENDERING_UPDATE");
    }

    fn unprepare(&mut self, app: &mut App) {
        serialize_to_file(&self.config, self.config.get_filepath().as_path());

        let update_phase: &mut PhaseWithSystems = app.get_phase_mut(VIEWER_UPDATE_PHASE);
        update_phase.remove_system(&self.updater_id);
        app.destroy_phase(VIEWER_UPDATE_PHASE);
    }
}
