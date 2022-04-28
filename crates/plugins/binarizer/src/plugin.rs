use inox_core::{define_plugin, ContextRc, Plugin, System, SystemId};

use crate::Binarizer;

#[repr(C)]
#[derive(Default)]
pub struct BinarizerPlugin {
    updater_id: SystemId,
}
define_plugin!(BinarizerPlugin);

impl Plugin for BinarizerPlugin {
    fn create(_context: &ContextRc) -> Self {
        BinarizerPlugin::default()
    }
    fn name(&self) -> &str {
        "inox_binarizer"
    }
    fn prepare(&mut self, context: &ContextRc) {
        let system = Binarizer::new(
            context,
            inox_resources::Data::data_raw_folder(),
            inox_resources::Data::data_folder(),
        );
        self.updater_id = Binarizer::system_id();
        context.add_system(inox_core::Phases::PreUpdate, system, None);
    }

    fn unprepare(&mut self, context: &ContextRc) {
        context.remove_system(inox_core::Phases::PreUpdate, &self.updater_id);
    }
    fn load_config(&mut self, _context: &ContextRc) {}
}
