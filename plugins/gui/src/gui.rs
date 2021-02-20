use super::config::*;
use nrg_core::*;
use nrg_serialize::*;
use std::any::type_name;

use super::gui_updater::*;

const GUI_UPDATE_PHASE: &str = "GUI_UPDATE_PHASE";

#[repr(C)]
pub struct Gui {
    config: Config,
    updater_id: SystemId,
    renderer_id: SystemId,
}

impl Default for Gui {
    fn default() -> Self {
        println!("Created {} plugin", type_name::<Self>().to_string());
        Self {
            config: Config::default(),
            updater_id: SystemId::default(),
            renderer_id: SystemId::default(),
        }
    }
}

impl Drop for Gui {
    fn drop(&mut self) {
        println!("Destroyed {} plugin", type_name::<Self>().to_string());
    }
}

unsafe impl Send for Gui {}
unsafe impl Sync for Gui {}

impl Plugin for Gui {
    fn prepare<'a>(&mut self, scheduler: &mut Scheduler, shared_data: &mut SharedDataRw) {
        let path = self.config.get_filepath();
        deserialize(&mut self.config, path);

        let mut update_phase = PhaseWithSystems::new(GUI_UPDATE_PHASE);
        let system = GuiUpdater::new(shared_data, &self.config);
        self.updater_id = system.id();
        update_phase.add_system(system);
        scheduler.create_phase(update_phase);
    }

    fn unprepare(&mut self, scheduler: &mut Scheduler) {
        let path = self.config.get_filepath();
        serialize(&self.config, path);

        let update_phase: &mut PhaseWithSystems = scheduler.get_phase_mut(GUI_UPDATE_PHASE);
        update_phase.remove_system(&self.updater_id);
        scheduler.destroy_phase(GUI_UPDATE_PHASE);
    }
}
