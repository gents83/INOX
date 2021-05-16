use std::sync::{Arc, RwLock};

use nrg_core::*;
use nrg_graphics::Renderer;
use nrg_math::Vector2;
use nrg_platform::Window;
use nrg_resources::{ConfigBase, SharedData};
use nrg_serialize::*;

use crate::{config::*, update_system::UpdateSystem};

use super::rendering_system::*;

const RENDERING_THREAD: &str = "Worker1";
const RENDERING_UPDATE: &str = "RENDERING_UPDATE";
const RENDERING_PHASE: &str = "RENDERING_PHASE";

#[repr(C)]
pub struct GfxPlugin {
    config: Config,
    update_system_id: SystemId,
    rendering_system_id: SystemId,
}

impl Default for GfxPlugin {
    fn default() -> Self {
        nrg_profiler::register_thread_into_profiler_with_name!("GfxPlugin");
        Self {
            config: Config::default(),
            update_system_id: SystemId::default(),
            rendering_system_id: SystemId::default(),
        }
    }
}

unsafe impl Send for GfxPlugin {}
unsafe impl Sync for GfxPlugin {}

impl Plugin for GfxPlugin {
    fn prepare(&mut self, app: &mut App) {
        let path = self.config.get_filepath();
        deserialize_from_file(&mut self.config, path);

        let renderer = {
            let shared_data = app.get_shared_data();
            let window = SharedData::get_unique_resource::<Window>(&shared_data);
            let mut renderer = Renderer::new(
                window.get().get_handle(),
                self.config.vk_data.debug_validation_layers,
            );
            let size = Vector2::new(
                window.get().get_width() as _,
                window.get().get_heigth() as _,
            );
            renderer.set_viewport_size(size);
            renderer
        };
        let renderer = Arc::new(RwLock::new(renderer));

        let mut update_phase = PhaseWithSystems::new(RENDERING_UPDATE);
        let system = UpdateSystem::new(
            renderer.clone(),
            &app.get_shared_data(),
            &app.get_global_messenger(),
            &self.config,
        );
        self.update_system_id = system.id();

        let mut rendering_phase = PhaseWithSystems::new(RENDERING_PHASE);
        let rendering_system = RenderingSystem::new(renderer);
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
    }
}
