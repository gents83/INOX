use nrg_core::*;
use nrg_platform::Window;
use nrg_resources::{ConfigBase, ResourceId};
use nrg_serialize::*;

use crate::config::*;
use crate::window_system::*;

const MAIN_WINDOW_PHASE: &str = "MAIN_WINDOW_PHASE";

#[repr(C)]
pub struct MainWindow {
    config: Config,
    system_id: SystemId,
    window_id: ResourceId,
}

impl Default for MainWindow {
    fn default() -> Self {
        Self {
            config: Config::default(),
            system_id: SystemId::default(),
            window_id: INVALID_UID,
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
            Window::create(
                name.clone(),
                pos.x as _,
                pos.y as _,
                size.x as _,
                size.y as _,
                app.get_global_messenger(),
            )
        };
        self.window_id = app.get_shared_data().write().unwrap().add_resource(window);

        let mut update_phase = PhaseWithSystems::new(MAIN_WINDOW_PHASE);
        let system = WindowSystem::new(&mut app.get_shared_data());

        self.system_id = system.id();

        update_phase.add_system(system);
        app.create_phase(update_phase);
    }

    fn unprepare(&mut self, app: &mut App) {
        let path = self.config.get_filepath();
        serialize_to_file(&self.config, path);

        let update_phase: &mut PhaseWithSystems = app.get_phase_mut(MAIN_WINDOW_PHASE);
        update_phase.remove_system(&self.system_id);
        app.destroy_phase(MAIN_WINDOW_PHASE);

        let shared_data = app.get_shared_data();
        shared_data
            .write()
            .unwrap()
            .remove_resource::<Window>(self.window_id);
    }
}
