use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use nrg_core::{App, PhaseWithSystems};
use nrg_graphics::{rendering_system::RenderingSystem, update_system::UpdateSystem, Renderer};

use nrg_messenger::Message;
use nrg_platform::{Window, WindowEvent};

use crate::window_system::WindowSystem;

const RENDERING_THREAD: &str = "Worker1";
const RENDERING_UPDATE: &str = "RENDERING_UPDATE";
const RENDERING_PHASE: &str = "RENDERING_PHASE";
const MAIN_WINDOW_PHASE: &str = "MAIN_WINDOW_PHASE";

#[repr(C)]
#[derive(Default)]
pub struct Launcher {}

impl Launcher {
    pub fn prepare(&mut self, app: &mut App) {
        let window = {
            Window::create(
                "NRG".to_string(),
                0,
                0,
                0,
                0,
                PathBuf::from("").as_path(),
                app.get_global_messenger(),
            )
        };

        let renderer = Renderer::new(window.get_handle(), &app.get_shared_data(), false);
        let renderer = Arc::new(RwLock::new(renderer));

        let mut window_update_phase = PhaseWithSystems::new(MAIN_WINDOW_PHASE);
        let window_system = WindowSystem::new(window);

        window_update_phase.add_system(window_system);
        app.create_phase(window_update_phase);

        let mut rendering_update_phase = PhaseWithSystems::new(RENDERING_UPDATE);
        let render_update_system = UpdateSystem::new(
            renderer.clone(),
            &app.get_shared_data(),
            &app.get_global_messenger(),
            app.get_job_handler(),
        );

        let mut rendering_draw_phase = PhaseWithSystems::new(RENDERING_PHASE);
        let rendering_draw_system = RenderingSystem::new(renderer, &app.get_shared_data());

        rendering_update_phase.add_system(render_update_system);
        rendering_draw_phase.add_system(rendering_draw_system);

        app.create_phase(rendering_update_phase);
        app.create_phase_on_worker(rendering_draw_phase, RENDERING_THREAD);

        app.get_global_messenger()
            .read()
            .unwrap()
            .get_dispatcher()
            .write()
            .unwrap()
            .send(WindowEvent::RequestChangeVisible(true).as_boxed())
            .ok();
    }

    pub fn unprepare(&mut self, app: &mut App) {
        app.destroy_phase_on_worker(RENDERING_PHASE, RENDERING_THREAD);
        app.destroy_phase(RENDERING_UPDATE);
        app.destroy_phase(MAIN_WINDOW_PHASE);
    }
}
