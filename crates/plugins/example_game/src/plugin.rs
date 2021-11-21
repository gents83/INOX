use sabi_core::{define_plugin, App, PhaseWithSystems, Plugin, System, SystemId};

use crate::systems::example_game::ExampleGame;

const EXAMPLE_GAME_PHASE: &str = "EXAMPLE_GAME_PHASE";

#[repr(C)]
#[derive(Default)]
pub struct ExampleGamePlugin {
    updater_id: SystemId,
}
define_plugin!(ExampleGamePlugin);

impl Plugin for ExampleGamePlugin {
    fn name(&self) -> &str {
        "sabi_example_game"
    }
    fn prepare(&mut self, app: &mut App) {
        let mut update_phase = PhaseWithSystems::new(EXAMPLE_GAME_PHASE);
        let mut system = ExampleGame::new(app.get_global_messenger(), app.get_shared_data());
        self.updater_id = ExampleGame::id();
        system.read_config(self.name());
        update_phase.add_system(system);

        app.create_phase_before(update_phase, "RENDERING_UPDATE");
    }

    fn unprepare(&mut self, app: &mut App) {
        let update_phase: &mut PhaseWithSystems = app.get_phase_mut(EXAMPLE_GAME_PHASE);
        update_phase.remove_system(&self.updater_id);
        app.destroy_phase(EXAMPLE_GAME_PHASE);
    }
}
