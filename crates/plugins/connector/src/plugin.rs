use sabi_core::{define_plugin, App, PhaseWithSystems, Plugin, System, SystemId};

use crate::systems::connector::Connector;

const CONNECTOR_PHASE: &str = "CONNECTOR_PHASE";

#[repr(C)]
#[derive(Default)]
pub struct ConnectorPlugin {
    updater_id: SystemId,
}
define_plugin!(ConnectorPlugin);

impl Plugin for ConnectorPlugin {
    fn name(&self) -> &str {
        "sabi_connector"
    }
    fn prepare(&mut self, app: &mut App) {
        let mut update_phase = PhaseWithSystems::new(CONNECTOR_PHASE);
        let mut system = Connector::new(app.get_global_messenger());
        self.updater_id = Connector::id();
        system.read_config(self.name());
        update_phase.add_system(system);

        app.create_phase(update_phase);
    }

    fn unprepare(&mut self, app: &mut App) {
        let update_phase: &mut PhaseWithSystems = app.get_phase_mut(CONNECTOR_PHASE);
        update_phase.remove_system(&self.updater_id);
        app.destroy_phase(CONNECTOR_PHASE);
    }
}