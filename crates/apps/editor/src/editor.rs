use nrg_core::{define_plugin, App, PhaseWithSystems, Plugin, System, SystemId};
use nrg_graphics::DebugDrawerSystem;
use nrg_resources::ConfigBase;
use nrg_serialize::read_from_file;
use nrg_ui::UISystem;

use crate::{config::Config, editor_updater::EditorUpdater};

const EDITOR_UPDATE_PHASE: &str = "EDITOR_UPDATE_PHASE";

#[repr(C)]
pub struct Editor {
    updater_id: SystemId,
    debug_drawer_id: SystemId,
    ui_id: SystemId,
    renderer_id: SystemId,
}
define_plugin!(Editor);

impl Default for Editor {
    fn default() -> Self {
        Self {
            updater_id: SystemId::default(),
            debug_drawer_id: SystemId::default(),
            ui_id: SystemId::default(),
            renderer_id: SystemId::default(),
        }
    }
}

impl Plugin for Editor {
    fn name(&self) -> &str {
        "nrg_editor"
    }
    fn prepare(&mut self, app: &mut App) {
        let mut config = Config::default();
        config = read_from_file(config.get_filepath(self.name()).as_path());

        let mut update_phase = PhaseWithSystems::new(EDITOR_UPDATE_PHASE);
        let system = EditorUpdater::new(app.get_shared_data(), app.get_global_messenger(), config);
        self.updater_id = EditorUpdater::id();
        update_phase.add_system(system);

        let debug_drawer_system =
            DebugDrawerSystem::new(app.get_shared_data(), app.get_global_messenger());
        self.debug_drawer_id = DebugDrawerSystem::id();
        update_phase.add_system(debug_drawer_system);

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
        let update_phase: &mut PhaseWithSystems = app.get_phase_mut(EDITOR_UPDATE_PHASE);
        update_phase.remove_system(&self.updater_id);
        app.destroy_phase(EDITOR_UPDATE_PHASE);
    }
}
