use nrg_core::*;
use nrg_serialize::*;

use crate::config::*;

use super::rendering_system::*;

const RENDERING_PHASE: &str = "RENDERING_PHASE";

#[repr(C)]
pub struct GfxPlugin {
    config: Config,
    system_id: SystemId,
}

impl Default for GfxPlugin {
    fn default() -> Self {
        Self {
            config: Config::default(),
            system_id: SystemId::default(),
        }
    }
}

unsafe impl Send for GfxPlugin {}
unsafe impl Sync for GfxPlugin {}

impl Plugin for GfxPlugin {
    fn prepare<'a>(&mut self, scheduler: &mut Scheduler, shared_data: &mut SharedDataRw) {
        let path = self.config.get_filepath();
        deserialize_from_file(&mut self.config, path);

        let mut update_phase = PhaseWithSystems::new(RENDERING_PHASE);
        let system = RenderingSystem::new(shared_data, &self.config);

        self.system_id = system.id();

        update_phase.add_system(system);
        scheduler.create_phase(update_phase);
    }

    fn unprepare(&mut self, scheduler: &mut Scheduler) {
        let path = self.config.get_filepath();
        serialize_to_file(&self.config, path);

        let update_phase: &mut PhaseWithSystems = scheduler.get_phase_mut(RENDERING_PHASE);
        update_phase.remove_system(&self.system_id);
        scheduler.destroy_phase(RENDERING_PHASE);
    }
}
