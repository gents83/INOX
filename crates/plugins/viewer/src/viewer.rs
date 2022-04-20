use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use inox_core::{define_plugin, App, Plugin, System, SystemId, WindowSystem};

use inox_graphics::{
    rendering_system::RenderingSystem, update_system::UpdateSystem, DebugDrawerSystem, Renderer,
};
use inox_platform::Window;
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
        let window = {
            Window::create(
                "SABI".to_string(),
                0,
                0,
                0,
                0,
                PathBuf::from("").as_path(),
                app.get_context().message_hub(),
            )
        };
        let renderer = Renderer::new(
            window.get_handle(),
            app.get_context().shared_data(),
            app.get_context().message_hub(),
            false,
        );
        let renderer = Arc::new(RwLock::new(renderer));

        let window_system = WindowSystem::new(
            window,
            app.get_context().shared_data(),
            app.get_context().message_hub(),
        );

        let render_update_system = UpdateSystem::new(
            renderer.clone(),
            app.get_context().shared_data(),
            app.get_context().message_hub(),
        );

        let rendering_draw_system = RenderingSystem::new(renderer, app.get_job_handler());

        let debug_drawer_system = DebugDrawerSystem::new(
            app.get_context().shared_data(),
            app.get_context().message_hub(),
        );
        self.debug_drawer_id = DebugDrawerSystem::id();

        let ui_system = UISystem::new(
            app.get_context().shared_data(),
            app.get_context().message_hub(),
            app.get_job_handler(),
        );
        self.ui_id = UISystem::id();

        let system = ViewerSystem::new(app.get_context());
        self.updater_id = ViewerSystem::id();

        app.add_system(inox_core::Phases::PlatformUpdate, window_system);
        app.add_system_with_dependencies(
            inox_core::Phases::Render,
            render_update_system,
            &[RenderingSystem::id()],
        );
        app.add_system_with_dependencies(
            inox_core::Phases::Render,
            rendering_draw_system,
            &[UpdateSystem::id()],
        );
        app.add_system(inox_core::Phases::Update, system);
        app.add_system(inox_core::Phases::Update, ui_system);
        app.add_system(inox_core::Phases::Update, debug_drawer_system);
    }

    fn unprepare(&mut self, app: &mut App) {
        app.remove_system(inox_core::Phases::Update, &self.debug_drawer_id);
        app.remove_system(inox_core::Phases::Update, &self.ui_id);
        app.remove_system(inox_core::Phases::Update, &self.updater_id);

        app.remove_system(inox_core::Phases::PlatformUpdate, &WindowSystem::id());
        app.remove_system(inox_core::Phases::Render, &UpdateSystem::id());
        app.remove_system(inox_core::Phases::Render, &RenderingSystem::id());
    }
}
