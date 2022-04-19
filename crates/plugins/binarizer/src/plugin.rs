use inox_core::{define_plugin, App, Plugin, System, SystemId};

use crate::Binarizer;

#[repr(C)]
#[derive(Default)]
pub struct BinarizerPlugin {
    updater_id: SystemId,
}
define_plugin!(BinarizerPlugin);

impl Plugin for BinarizerPlugin {
    fn name(&self) -> &str {
        "inox_binarizer"
    }
    fn prepare(&mut self, app: &mut App) {
        let system = Binarizer::new(
            app.get_context(),
            inox_resources::Data::data_raw_folder(),
            inox_resources::Data::data_folder(),
        );
        self.updater_id = Binarizer::id();
        app.add_system(inox_core::Phases::PreUpdate, system);
    }

    fn unprepare(&mut self, app: &mut App) {
        app.remove_system(inox_core::Phases::PreUpdate, &self.updater_id);
    }
}
