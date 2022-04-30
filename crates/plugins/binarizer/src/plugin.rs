use inox_core::{define_plugin, ContextRc, Plugin, SystemId, SystemUID};
use inox_platform::{PLATFORM_TYPE_PC, PLATFORM_TYPE_WEB};
use inox_resources::{PC_FOLDER, WEB_FOLDER};

use crate::Binarizer;

#[repr(C)]
#[derive(Default)]
pub struct BinarizerPlugin {
    pc_id: SystemId,
    web_id: SystemId,
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
        let system = Binarizer::<PLATFORM_TYPE_PC>::new(
            context,
            inox_resources::Data::data_raw_folder(),
            inox_resources::Data::data_folder().join(PC_FOLDER),
        );
        self.pc_id = Binarizer::<PLATFORM_TYPE_PC>::system_id();
        context.add_system(inox_core::Phases::PreUpdate, system, None);

        let system = Binarizer::<PLATFORM_TYPE_WEB>::new(
            context,
            inox_resources::Data::data_raw_folder(),
            inox_resources::Data::data_folder().join(WEB_FOLDER),
        );
        self.web_id = Binarizer::<PLATFORM_TYPE_WEB>::system_id();
        context.add_system(inox_core::Phases::PreUpdate, system, None);
    }

    fn unprepare(&mut self, context: &ContextRc) {
        context.remove_system(inox_core::Phases::PreUpdate, &self.pc_id);
        context.remove_system(inox_core::Phases::PreUpdate, &self.web_id);
    }
    fn load_config(&mut self, _context: &ContextRc) {}
}
