use std::path::PathBuf;
use super::scheduler::*;
use super::plugin::*;
use super::plugin_manager::*;
use super::phase::*;
use super::shared_data::*;


pub struct App {
    scheduler: Scheduler,
    plugin_manager: PluginManager,
    shared_data: SharedData,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.plugin_manager.release(&mut self.shared_data, &mut self.scheduler);
    }
}

impl App {
    pub fn new() -> Self {  
        Self {
            scheduler: Scheduler::new(),
            plugin_manager: PluginManager::new(),
            shared_data: SharedData::default(),
        }
    }

    pub fn run_once(&mut self) {
        self.scheduler.run_once();    
        self.plugin_manager.update(&mut self.shared_data, &mut self.scheduler);        
    }

    pub fn run(&mut self) {            
        loop 
        {                
            let is_ended = false;
            self.run_once();
            if is_ended
            {
                break;
            }
        }
    }

    pub fn create_phase<T: Phase>(&mut self, phase: T) -> &mut Self {
        self.scheduler.create_phase(phase);
        self
    }

    pub fn create_phase_with_systems(&mut self, phase_name: &str) -> &mut Self {
        self.scheduler.create_phase_with_systems(phase_name);
        self
    }

    pub fn get_phase<S: Phase>(&mut self, phase_name:&str) -> &S {
        self.scheduler.get_phase(phase_name)
    }

    pub fn get_phase_mut<S: Phase>(&mut self, phase_name:&str) -> &mut S {
        self.scheduler.get_phase_mut(phase_name)
    }
    
    pub fn add_plugin(&mut self, lib_path: PathBuf) -> PluginId {
        self.plugin_manager.add_plugin(lib_path, &mut self.shared_data, &mut self.scheduler)
    }

    pub fn remove_plugin(&mut self, plugin_id: &PluginId) {
        self.plugin_manager.remove_plugin(plugin_id, &mut self.shared_data, &mut self.scheduler)
    }
}