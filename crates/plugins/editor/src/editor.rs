use sabi_core::{define_plugin, App, PhaseWithSystems, Plugin, System, SystemId};
use sabi_graphics::DebugDrawerSystem;
use sabi_resources::ConfigBase;
use sabi_serialize::read_from_file;
use sabi_ui::UISystem;

use crate::{config::Config, editor_updater::EditorUpdater};

const EDITOR_UPDATE_PHASE: &str = "EDITOR_UPDATE_PHASE";

#[repr(C)]
#[derive(Default)]
pub struct Editor {
    updater_id: SystemId,
    debug_drawer_id: SystemId,
    ui_id: SystemId,
    renderer_id: SystemId,
}
define_plugin!(Editor);

impl Plugin for Editor {
    fn name(&self) -> &str {
        "sabi_editor"
    }
    fn prepare(&mut self, app: &mut App) {
        app.get_shared_data().register_serializable_type::<Config>();

        let mut config = Config::default();
        config = read_from_file(
            config.get_filepath(self.name()).as_path(),
            &app.get_shared_data().serializable_registry(),
        );

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

        app.get_shared_data()
            .unregister_serializable_type::<Config>();
    }
}
