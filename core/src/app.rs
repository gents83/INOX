use nrg_platform::*;

use crate::plugins::plugin::*;
use crate::plugins::plugin_manager::*;
use crate::resources::shared_data::*;
use crate::schedule::phase::*;
use crate::schedule::scheduler::*;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

pub struct App {
    frame_count: u64,
    plugin_manager: PluginManager,
    scheduler: Scheduler,
    shared_data: SharedDataRw,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.scheduler.uninit();
        let mut data = self.shared_data.write().unwrap();
        data.process_pending_requests();
        self.plugin_manager.release(&mut self.scheduler);
    }
}

impl App {
    pub fn new() -> Self {
        let shared_data = Arc::new(RwLock::new(SharedData::default()));
        {
            let mut data = shared_data.write().unwrap();
            data.add_resource(EventsRw::default());
        }
        Self {
            frame_count: 0,
            scheduler: Scheduler::new(),
            plugin_manager: PluginManager::new(),
            shared_data,
        }
    }

    pub fn get_shared_data(&self) -> SharedDataRw {
        self.shared_data.clone()
    }

    pub fn run_once(&mut self) -> bool {
        let can_continue = self.scheduler.run_once();
        self.shared_data.write().unwrap().process_pending_requests();
        self.plugin_manager
            .update(&mut self.shared_data, &mut self.scheduler);

        let data = self.shared_data.write().unwrap();
        let events_rw = &mut *data.get_unique_resource_mut::<EventsRw>();
        let mut events = events_rw.write().unwrap();
        events.update(self.frame_count);

        self.frame_count += 1;

        can_continue
    }

    pub fn run(&mut self) {
        loop {
            let can_continue = self.run_once();
            if !can_continue {
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

    pub fn get_phase<S: Phase>(&mut self, phase_name: &str) -> &S {
        self.scheduler.get_phase(phase_name)
    }

    pub fn get_phase_mut<S: Phase>(&mut self, phase_name: &str) -> &mut S {
        self.scheduler.get_phase_mut(phase_name)
    }
    pub fn add_plugin(&mut self, lib_path: PathBuf) {
        self.plugin_manager
            .add_plugin(lib_path, &mut self.shared_data, &mut self.scheduler)
    }

    pub fn remove_plugin(&mut self, plugin_id: &PluginId) {
        self.plugin_manager
            .remove_plugin(plugin_id, &mut self.scheduler)
    }
}
