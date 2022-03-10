use inox_core::{define_plugin, App, Plugin, System, SystemId};

use crate::systems::connector::Connector;

#[repr(C)]
#[derive(Default)]
pub struct ConnectorPlugin {
    updater_id: SystemId,
}
define_plugin!(ConnectorPlugin);

impl Plugin for ConnectorPlugin {
    fn name(&self) -> &str {
        "inox_connector"
    }
    fn prepare(&mut self, app: &mut App) {
        let mut system = Connector::new(app.get_context());
        self.updater_id = Connector::id();
        system.read_config(self.name());

        app.add_system(inox_core::Phases::Update, system);
    }

    fn unprepare(&mut self, app: &mut App) {
        app.remove_system(inox_core::Phases::Update, &self.updater_id);
    }
}
