use inox_core::{define_plugin, ContextRc, Plugin, System, SystemId};

use crate::systems::connector::Connector;

#[repr(C)]
#[derive(Default)]
pub struct ConnectorPlugin {
    updater_id: SystemId,
}
define_plugin!(ConnectorPlugin);

impl Plugin for ConnectorPlugin {
    fn create(_context: &ContextRc) -> Self {
        ConnectorPlugin::default()
    }
    fn name(&self) -> &str {
        "inox_connector"
    }
    fn prepare(&mut self, context: &ContextRc) {
        let system = Connector::new(context);
        self.updater_id = system.id();
        context.add_system(inox_core::Phases::Update, system, None);
    }

    fn unprepare(&mut self, context: &ContextRc) {
        context.remove_system(inox_core::Phases::Update, &self.updater_id);
    }
    fn load_config(&mut self, _context: &ContextRc) {}
}
