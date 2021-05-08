use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
};

use nrg_events::EventsRw;
use nrg_platform::{InputState, Key, KeyEvent};
use nrg_resources::SharedDataRw;

use crate::{Job, Phase, PluginId, PluginManager, Scheduler, Worker};

pub struct App {
    frame_count: u64,
    is_profiling: bool,
    shared_data: SharedDataRw,
    plugin_manager: PluginManager,
    scheduler: Scheduler,
    workers: HashMap<String, Worker>,
    sender: Sender<Job>,
    receiver: Arc<Mutex<Receiver<Job>>>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for App {
    fn drop(&mut self) {
        for (_name, w) in self.workers.iter_mut() {
            w.stop();
        }
        if self.is_profiling {
            nrg_profiler::write_profile_file!();
        }

        self.scheduler.uninit();
        self.shared_data.write().unwrap().clear();

        let plugins_to_remove = self.plugin_manager.release();
        self.update_plugins(plugins_to_remove, false);
    }
}

impl App {
    pub fn new() -> Self {
        nrg_profiler::create_profiler!();
        let shared_data = SharedDataRw::default();
        {
            let mut data = shared_data.write().unwrap();
            data.add_resource(EventsRw::default());
        }
        let (sender, receiver) = channel();

        let mut app = Self {
            frame_count: 0,
            is_profiling: false,
            scheduler: Scheduler::new(),
            plugin_manager: PluginManager::new(),
            workers: HashMap::new(),
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            shared_data,
        };

        app.add_worker("Worker1");
        app.add_worker("Worker2");
        app.add_worker("Worker3");
        app.add_worker("Worker4");
        app.add_worker("Worker5");

        app
    }

    fn update_plugins(&mut self, plugins_to_remove: Vec<PluginId>, reload: bool) {
        for id in plugins_to_remove.iter() {
            if let Some(plugin_data) = self.plugin_manager.remove_plugin(id) {
                let lib_path = plugin_data.original_path.clone();
                PluginManager::clear_plugin_data(plugin_data, self);
                if reload {
                    let reloaded_plugin_data = PluginManager::create_plugin_data(lib_path, self);
                    self.plugin_manager.add_plugin(reloaded_plugin_data);
                }
            }
        }
    }

    pub fn run_once(&mut self) -> bool {
        nrg_profiler::scoped_profile!("app::run_frame");

        let (can_continue, jobs) = self.scheduler.run_once();
        for j in jobs {
            let res = self.sender.send(j);
            if res.is_err() {
                panic!("Failed to add job to execution queue");
            }
        }

        let plugins_to_remove = self.plugin_manager.update();
        self.update_plugins(plugins_to_remove, true);

        {
            let data = self.shared_data.read().unwrap();
            let events_rw = &mut *data.get_unique_resource_mut::<EventsRw>();
            let mut events = events_rw.write().unwrap();
            self.frame_count += 1;
            events.update(self.frame_count);
        }

        can_continue
    }

    pub fn run(&mut self) {
        loop {
            let can_continue = self.run_once();

            self.manage_hotkeys();

            if !can_continue {
                break;
            }
        }
    }

    fn manage_hotkeys(&mut self) {
        let data = self.shared_data.read().unwrap();
        let events_rw = &mut *data.get_unique_resource_mut::<EventsRw>();
        let events = events_rw.read().unwrap();
        if let Some(key_events) = events.read_all_events::<KeyEvent>() {
            for event in key_events.iter() {
                if event.code == Key::F9 && event.state == InputState::JustPressed {
                    if !self.is_profiling {
                        nrg_profiler::start_profiler!();
                        self.is_profiling = true;
                    } else {
                        self.is_profiling = false;
                        nrg_profiler::stop_profiler!();
                        nrg_profiler::write_profile_file!();
                    }
                }
            }
        }
    }
    pub fn add_plugin(&mut self, lib_path: PathBuf) {
        let plugin_data = PluginManager::create_plugin_data(lib_path, self);
        self.plugin_manager.add_plugin(plugin_data);
    }

    pub fn remove_plugin(&mut self, plugin_id: &PluginId) {
        if let Some(plugin_data) = self.plugin_manager.remove_plugin(plugin_id) {
            PluginManager::clear_plugin_data(plugin_data, self);
        }
    }
    fn add_worker(&mut self, name: &'static str) -> &mut Worker {
        let key = String::from(name);
        let w = self.workers.entry(key).or_insert_with(Worker::default);
        if !w.is_started() {
            w.start(name, self.receiver.clone());
        }
        w
    }
    fn get_worker(&mut self, name: &str) -> &mut Worker {
        let key = String::from(name);
        self.workers.get_mut(&key).unwrap()
    }

    pub fn get_shared_data(&self) -> SharedDataRw {
        self.shared_data.clone()
    }
    pub fn create_phase_on_worker<P: Phase>(&mut self, phase: P, name: &'static str) {
        let w = self.add_worker(name);
        w.create_phase(phase);
    }
    pub fn destroy_phase_on_worker(&mut self, phase_name: &str, name: &str) {
        let w = self.get_worker(name);
        w.destroy_phase(phase_name);
    }
    pub fn create_phase<P: Phase>(&mut self, phase: P) {
        self.scheduler.create_phase(phase);
    }
    pub fn destroy_phase(&mut self, phase_name: &str) {
        self.scheduler.destroy_phase(phase_name);
    }
    pub fn get_phase<P: Phase>(&mut self, phase_name: &str) -> &P {
        self.scheduler.get_phase(phase_name)
    }
    pub fn get_phase_mut<P: Phase>(&mut self, phase_name: &str) -> &mut P {
        self.scheduler.get_phase_mut(phase_name)
    }
}
