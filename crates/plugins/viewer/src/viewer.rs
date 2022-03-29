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
        let debug_drawer_system = DebugDrawerSystem::new(
            app.get_context().shared_data(),
            app.get_context().message_hub(),
        );
        self.debug_drawer_id = DebugDrawerSystem::id();

        let mut ui_system = UISystem::new(
            app.get_context().shared_data(),
            app.get_context().message_hub(),
            app.get_job_handler(),
        );
        ui_system.read_config(self.name());
        self.ui_id = UISystem::id();

        let system = ViewerSystem::new(app.get_context());
        self.updater_id = ViewerSystem::id();

        app.add_system(inox_core::Phases::Update, system);
        app.add_system_with_dependencies(
            inox_core::Phases::Update,
            ui_system,
            &[ViewerSystem::id()],
        );
        app.add_system_with_dependencies(
            inox_core::Phases::Update,
            debug_drawer_system,
            &[ViewerSystem::id()],
        );
    }

    fn unprepare(&mut self, app: &mut App) {
        app.remove_system(inox_core::Phases::Update, &self.debug_drawer_id);
        app.remove_system(inox_core::Phases::Update, &self.ui_id);
        app.remove_system(inox_core::Phases::Update, &self.updater_id);
    }
}
