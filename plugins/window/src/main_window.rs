use nrg_app::*;

use super::window_system::*;

const MAIN_WINDOW_PHASE:&str = "MAIN_WINDOW_PHASE";

#[repr(C)]
pub struct MainWindow {
    system_id: SystemId,
}

impl Default for MainWindow {
    fn default() -> Self {
        println!("Created {} plugin", ::std::any::type_name::<MainWindow>().to_string());
        Self{
            system_id: SystemId::default(),
        }
    }
}

impl Drop for MainWindow {
    fn drop(&mut self) {
        println!("Destroyed {} plugin", ::std::any::type_name::<MainWindow>().to_string());
    }
}

unsafe impl Send for MainWindow {}
unsafe impl Sync for MainWindow {}

impl Plugin for MainWindow {
    fn prepare(&mut self, scheduler: &mut Scheduler, _shared_data: &mut SharedData) {    
        println!("Prepare {} plugin", ::std::any::type_name::<MainWindow>().to_string());
        
        let mut update_phase = PhaseWithSystems::new(MAIN_WINDOW_PHASE);
        let system = WindowSystem::new(String::from("NRG"));

        self.system_id = system.id();

        update_phase.add_system(system);
        scheduler.create_phase(update_phase); 
    }
    
    fn unprepare(&mut self, scheduler: &mut Scheduler, _shared_data: &mut SharedData) {   
        let update_phase:&mut PhaseWithSystems = scheduler.get_phase_mut(MAIN_WINDOW_PHASE);
        update_phase.remove_system(&self.system_id);
        scheduler.destroy_phase(MAIN_WINDOW_PHASE);
        println!("Unprepare {} plugin", ::std::any::type_name::<MainWindow>().to_string());
    }
}