use nrg_core::{define_plugin, App, PluginHolder, PluginId, System, SystemId};
use nrg_core::{PhaseWithSystems, Plugin};
use nrg_graphics::rendering_system::RenderingSystem;
use nrg_graphics::update_system::UpdateSystem;
use nrg_graphics::Renderer;
use nrg_math::Vector2;
use nrg_platform::Window;
use nrg_resources::ConfigBase;
use nrg_serialize::{deserialize_from_file, serialize_to_file};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use crate::config::Config;
use crate::window_system::WindowSystem;
const RENDERING_THREAD: &str = "Worker1";
const RENDERING_UPDATE: &str = "RENDERING_UPDATE";
const RENDERING_PHASE: &str = "RENDERING_PHASE";
const MAIN_WINDOW_PHASE: &str = "MAIN_WINDOW_PHASE";

#[repr(C)]
pub struct MainWindow {
    config: Config,
    system_id: SystemId,
    update_system_id: SystemId,
    rendering_system_id: SystemId,
}
define_plugin!(MainWindow);

impl Default for MainWindow {
    fn default() -> Self {
        Self {
            config: Config::default(),
            system_id: SystemId::default(),
            update_system_id: SystemId::default(),
            rendering_system_id: SystemId::default(),
        }
    }
}

unsafe impl Send for MainWindow {}
unsafe impl Sync for MainWindow {}

impl Plugin for MainWindow {
    fn prepare(&mut self, app: &mut App) {
        let path = self.config.get_filepath();
        deserialize_from_file(&mut self.config, path);

        let window = {
            let pos = self.config.get_position();
            let size = self.config.get_resolution();
            let name = self.config.get_name();
            let icon = self.config.get_icon();
            Window::create(
                name.clone(),
                pos.x as _,
                pos.y as _,
                size.x as _,
                size.y as _,
                PathBuf::from(icon).as_path(),
                app.get_global_messenger(),
            )
        };

        let renderer = {
            let mut renderer = Renderer::new(
                window.get_handle(),
                self.config.is_debug_validation_layers_enabled(),
            );
            let size = Vector2::new(window.get_width() as _, window.get_heigth() as _);
            renderer.set_viewport_size(size);
            renderer
        };
        let renderer = Arc::new(RwLock::new(renderer));

        let mut update_phase = PhaseWithSystems::new(MAIN_WINDOW_PHASE);
        let system = WindowSystem::new(window);

        self.system_id = system.id();

        update_phase.add_system(system);
        app.create_phase(update_phase);

        let mut update_phase = PhaseWithSystems::new(RENDERING_UPDATE);
        let system = UpdateSystem::new(
            renderer.clone(),
            &app.get_shared_data(),
            &app.get_global_messenger(),
            app.get_job_handler(),
        );
        self.update_system_id = system.id();

        let mut rendering_phase = PhaseWithSystems::new(RENDERING_PHASE);
        let rendering_system = RenderingSystem::new(renderer, &app.get_shared_data());
        self.rendering_system_id = rendering_system.id();

        update_phase.add_system(system);
        rendering_phase.add_system(rendering_system);

        app.create_phase(update_phase);
        app.create_phase_on_worker(rendering_phase, RENDERING_THREAD);
    }

    fn unprepare(&mut self, app: &mut App) {
        let path = self.config.get_filepath();
        serialize_to_file(&self.config, path);

        app.destroy_phase_on_worker(RENDERING_PHASE, RENDERING_THREAD);
        app.destroy_phase(RENDERING_UPDATE);
        app.destroy_phase(MAIN_WINDOW_PHASE);
    }
}
