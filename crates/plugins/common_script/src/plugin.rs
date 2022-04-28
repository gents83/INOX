use inox_core::{define_plugin, ContextRc, Plugin, SystemId};

#[repr(C)]
#[derive(Default)]
pub struct CommonScriptPlugin {
    updater_id: SystemId,
}
define_plugin!(CommonScriptPlugin);

impl Plugin for CommonScriptPlugin {
    fn create(_context: &ContextRc) -> Self {
        CommonScriptPlugin::default()
    }
    fn name(&self) -> &str {
        "inox_common_script"
    }
    fn prepare(&mut self, context: &ContextRc) {
        inox_nodes::register_nodes(context.shared_data());
        crate::logic_nodes::register_nodes(context.shared_data());
    }

    fn unprepare(&mut self, context: &ContextRc) {
        crate::logic_nodes::unregister_nodes(context.shared_data());
        inox_nodes::unregister_nodes(context.shared_data());
    }
    fn load_config(&mut self, _context: &ContextRc) {}
}
