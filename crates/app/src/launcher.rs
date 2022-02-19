use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use inox_core::{
    App, PfnCreatePlugin, PfnDestroyPlugin, PfnPreparePlugin, PfnUnpreparePlugin, PhaseWithSystems,
};
use inox_graphics::{
    rendering_system::{RenderingSystem, RENDERING_PHASE},
    update_system::{UpdateSystem, RENDERING_UPDATE},
    Renderer,
};

use inox_messenger::MessageHubRc;
use inox_platform::Window;
use inox_profiler::debug_log;
use inox_resources::SharedDataRc;

use crate::window_system::WindowSystem;

const MAIN_WINDOW_PHASE: &str = "MAIN_WINDOW_PHASE";

#[derive(Default)]
pub struct Launcher {
    app: Arc<RwLock<App>>,
}

impl Launcher {
    pub fn shared_data(&self) -> SharedDataRc {
        self.app.read().unwrap().get_shared_data().clone()
    }
    pub fn message_hub(&self) -> MessageHubRc {
        self.app.read().unwrap().get_message_hub().clone()
    }

    pub fn prepare(&self) {
        debug_log!("Preparing launcher");

        let app = &mut self.app.write().unwrap();
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
        let window_system = WindowSystem::new(window, app.get_shared_data(), app.get_message_hub());

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
        app.create_phase(rendering_draw_phase);
    }

    pub fn start(&self) {
        let app = &mut self.app.write().unwrap();

        debug_log!("Starting app");

        app.start();
    }

    pub fn add_dynamic_plugin(&self, name: &str, path: &Path) {
        let app = &mut self.app.write().unwrap();
        Self::read_config(app, name);
        app.add_dynamic_plugin(path);
    }

    pub fn add_static_plugin(
        &self,
        name: &str,
        fn_c: PfnCreatePlugin,
        fn_p: PfnPreparePlugin,
        fn_u: PfnUnpreparePlugin,
        fn_d: PfnDestroyPlugin,
    ) {
        let app = &mut self.app.write().unwrap();
        Self::read_config(app, name);

        if let Some(fn_c) = fn_c {
            let mut plugin_holder = unsafe { fn_c() };
            plugin_holder.prepare_fn = fn_p;
            plugin_holder.unprepare_fn = fn_u;
            plugin_holder.destroy_fn = fn_d;
            app.add_static_plugin(plugin_holder);
        }
    }

    fn read_config(app: &mut App, plugin_name: &str) {
        debug_log!("Reading launcher configs");

        let phase = app.get_phase_mut::<PhaseWithSystems>(MAIN_WINDOW_PHASE);
        if let Some(window_system) = phase.get_system_mut::<WindowSystem>() {
            window_system.read_config(plugin_name);
        }

        let phase = app.get_phase_mut::<PhaseWithSystems>(RENDERING_UPDATE);
        if let Some(window_system) = phase.get_system_mut::<UpdateSystem>() {
            window_system.read_config(plugin_name);
        }
    }

    pub fn update(&self) -> bool {
        self.app.write().unwrap().run()
    }
}

impl Drop for Launcher {
    fn drop(&mut self) {
        debug_log!("Dropping launcher");
        let app = &mut self.app.write().unwrap();
        app.destroy_phase(RENDERING_PHASE);
        app.destroy_phase(RENDERING_UPDATE);
        app.destroy_phase(MAIN_WINDOW_PHASE);
    }
}
