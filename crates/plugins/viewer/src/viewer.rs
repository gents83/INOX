use inox_core::{define_plugin, App, Plugin, System, SystemId};

use inox_graphics::DebugDrawerSystem;
use inox_ui::UISystem;

use crate::systems::viewer_system::ViewerSystem;

#[repr(C)]
#[derive(Default)]
pub struct Viewer {
    updater_id: SystemId,
    debug_drawer_id: SystemId,
    ui_id: SystemId,
}
define_plugin!(Viewer);

impl Plugin for Viewer {
    fn name(&self) -> &str {
        "inox_viewer"
    }
    fn prepare(&mut self, app: &mut App) {
        let system = ViewerSystem::new(app.get_shared_data(), app.get_message_hub());
        self.updater_id = ViewerSystem::id();

        let debug_drawer_system =
            DebugDrawerSystem::new(app.get_shared_data(), app.get_message_hub());
        self.debug_drawer_id = DebugDrawerSystem::id();

        let mut ui_system = UISystem::new(
            app.get_shared_data(),
            app.get_message_hub(),
            app.get_job_handler(),
        );
        ui_system.read_config(self.name());
        self.ui_id = UISystem::id();

        app.add_system(inox_core::Phases::Update, system);
        app.add_system(inox_core::Phases::PreRender, ui_system);
        app.add_system(inox_core::Phases::PreRender, debug_drawer_system);
    }

    fn unprepare(&mut self, app: &mut App) {
        app.remove_system(inox_core::Phases::PreRender, &self.debug_drawer_id);
        app.remove_system(inox_core::Phases::PreRender, &self.ui_id);
        app.remove_system(inox_core::Phases::Update, &self.updater_id);
    }
}
