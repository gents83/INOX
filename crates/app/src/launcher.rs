use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use sabi_core::{App, PhaseWithSystems};
use sabi_graphics::{
    rendering_system::{RenderingSystem, RENDERING_PHASE},
    update_system::{UpdateSystem, RENDERING_UPDATE},
    Renderer,
};

use sabi_platform::Window;

use crate::window_system::WindowSystem;

const RENDERING_THREAD: &str = "Worker1";
const MAIN_WINDOW_PHASE: &str = "MAIN_WINDOW_PHASE";

#[repr(C)]
#[derive(Default)]
pub struct Launcher {}

impl Launcher {
    pub fn prepare(&mut self, app: &mut App) {
        let window = {
            Window::create(
                "SABI".to_string(),
                0,
                0,
                0,
                0,
                PathBuf::from("").as_path(),
                app.get_message_hub(),
            )
        };

        let renderer = Renderer::new(window.get_handle(), app.get_shared_data(), false);
        let renderer = Arc::new(RwLock::new(renderer));

        let mut window_update_phase = PhaseWithSystems::new(MAIN_WINDOW_PHASE);
        let window_system = WindowSystem::new(window, app.get_message_hub());

        window_update_phase.add_system(window_system);
        app.create_phase(window_update_phase);

        let mut rendering_update_phase = PhaseWithSystems::new(RENDERING_UPDATE);
        let render_update_system = UpdateSystem::new(
            renderer.clone(),
            app.get_shared_data(),
            app.get_message_hub(),
            app.get_job_handler(),
        );

        let mut rendering_draw_phase = PhaseWithSystems::new(RENDERING_PHASE);
        let rendering_draw_system = RenderingSystem::new(
            renderer,
            app.get_shared_data(),
            app.get_message_hub(),
            app.get_job_handler(),
        );

        rendering_update_phase.add_system(render_update_system);
        rendering_draw_phase.add_system(rendering_draw_system);

        app.create_phase(rendering_update_phase);
        app.create_phase_on_worker(rendering_draw_phase, RENDERING_THREAD);
    }

    pub fn unprepare(&mut self, app: &mut App) {
        app.destroy_phase_on_worker(RENDERING_PHASE, RENDERING_THREAD);
        app.destroy_phase(RENDERING_UPDATE);
        app.destroy_phase(MAIN_WINDOW_PHASE);
    }

    pub fn read_config(&mut self, app: &mut App, plugin_name: &str) {
        let phase = app.get_phase_mut::<PhaseWithSystems>(MAIN_WINDOW_PHASE);
        if let Some(window_system) = phase.get_system_mut::<WindowSystem>() {
            window_system.read_config(plugin_name);
        }

        let phase = app.get_phase_mut::<PhaseWithSystems>(RENDERING_UPDATE);
        if let Some(window_system) = phase.get_system_mut::<UpdateSystem>() {
            window_system.read_config(plugin_name);
        }
    }
}
