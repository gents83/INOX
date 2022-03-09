use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        mpsc::{channel, Receiver},
        Arc, Mutex, RwLock,
    },
};

use inox_messenger::{Listener, MessageHubRc};
use inox_platform::{InputState, Key, KeyEvent, WindowEvent};
use inox_resources::{DeserializeFunction, SharedData, SharedDataRc};

use inox_uid::generate_uid_from_string;

use crate::{
    Job, JobHandler, JobHandlerRw, Phases, PluginHolder, PluginId, PluginManager, Scheduler,
    System, SystemId, Worker,
};

#[cfg(target_arch = "wasm32")]
const NUM_WORKER_THREADS: usize = 0;
#[cfg(all(not(target_arch = "wasm32")))]
const NUM_WORKER_THREADS: usize = 5;

pub struct App {
    is_profiling: bool,
    is_enabled: bool,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    listener: Listener,
    plugin_manager: PluginManager,
    scheduler: Scheduler,
    workers: HashMap<String, Worker>,
    job_handler: Arc<RwLock<JobHandler>>,
    receiver: Arc<Mutex<Receiver<Job>>>,
}

impl Default for App {
    fn default() -> Self {
        inox_profiler::create_profiler!();

        let (sender, receiver) = channel();

        let message_hub = MessageHubRc::default();
        let listener = Listener::new(&message_hub);

        listener.register::<KeyEvent>().register::<WindowEvent>();

        Self {
            is_enabled: true,
            is_profiling: false,
            scheduler: Scheduler::new(),
            plugin_manager: PluginManager::default(),
            workers: HashMap::new(),
            job_handler: JobHandler::new(sender),
            receiver: Arc::new(Mutex::new(receiver)),
            shared_data: SharedDataRc::default(),
            message_hub,
            listener,
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.stop_worker_threads();

        if self.is_profiling {
            inox_profiler::write_profile_file!();
        }

        self.scheduler.uninit();

        let plugins_to_remove = self.plugin_manager.release();
        self.update_dynamic_plugins(plugins_to_remove);
    }
}

impl App {
    pub fn start(&mut self) -> &mut Self {
        self.setup_worker_threads();
        self.scheduler.start();
        self
    }

    fn setup_worker_threads(&mut self) {
        if NUM_WORKER_THREADS > 0 {
            for i in 1..NUM_WORKER_THREADS + 1 {
                self.add_worker(format!("Worker{}", i).as_str());
            }
        }
    }

    fn stop_worker_threads(&mut self) {
        for (_name, w) in self.workers.iter_mut() {
            w.stop();
        }
        self.job_handler.write().unwrap().clear_pending_jobs();
    }

    fn update_dynamic_plugins(&mut self, plugins_to_remove: Vec<PluginId>) -> Vec<PathBuf> {
        let mut plugins_to_reload = Vec::new();
        for id in plugins_to_remove.iter() {
            if let Some(plugin_data) = self.plugin_manager.remove_dynamic_plugin(id) {
                let lib_path = plugin_data.original_path.clone();
                PluginManager::clear_plugin_data(plugin_data, self);
                plugins_to_reload.push(lib_path);
            }
        }
        plugins_to_reload
    }

    fn reload_dynamic_plugins(&mut self, plugins_to_reload: Vec<PathBuf>) {
        for lib_path in plugins_to_reload.into_iter() {
            let reloaded_plugin_data = PluginManager::create_plugin_data(lib_path.as_path(), self);
            self.plugin_manager.add_dynamic_plugin(reloaded_plugin_data);
        }
    }

    fn update_events(&mut self) {
        inox_profiler::scoped_profile!("app::update_events");

        let mut is_profiling = self.is_profiling;
        let mut is_enabled = self.is_enabled;

        self.listener.process_messages(|e: &KeyEvent| {
            if e.code == Key::F9 && e.state == InputState::JustPressed {
                if !is_profiling {
                    inox_profiler::start_profiler!();
                    is_profiling = true;
                } else {
                    is_profiling = false;
                    inox_profiler::stop_profiler!();
                    inox_profiler::write_profile_file!();
                }
            }
        });
        self.listener.process_messages(|e: &WindowEvent| match e {
            WindowEvent::Show => {
                is_enabled = true;
            }
            WindowEvent::Hide => {
                is_enabled = false;
            }
            _ => {}
        });
        self.shared_data
            .handle_events(|load_fn: Box<dyn DeserializeFunction>| {
                let shared_data = self.shared_data.clone();
                let message_hub = self.message_hub.clone();
                let job_name = "Load Event".to_string();
                let load_event_category = generate_uid_from_string("LOAD_EVENT_CATEGORY");
                self.job_handler.write().unwrap().add_job(
                    &load_event_category,
                    job_name.as_str(),
                    move || load_fn(&shared_data, &message_hub),
                );
            });

        //flush messages between frames
        self.message_hub.flush();

        self.is_profiling = is_profiling;

        self.update_workers(is_enabled);
        self.is_enabled = is_enabled;
    }

    fn update_workers(&mut self, is_enabled: bool) {
        if NUM_WORKER_THREADS == 0 {
            //no workers - need to handle events ourself
            let recv = self.receiver.lock().unwrap();
            if let Ok(job) = recv.try_recv() {
                drop(recv);
                job.execute();
            }
        }
        if !self.is_enabled && !is_enabled {
            self.stop_worker_threads();
        } else if !self.is_enabled && is_enabled {
            self.setup_worker_threads();
        }
    }

    pub fn run(&mut self) -> bool {
        inox_profiler::scoped_profile!("app::run_frame");

        let can_continue = self.scheduler.run_once(self.is_enabled, &self.job_handler);

        self.update_events();

        if !self.is_enabled {
            let plugins_to_remove = self.plugin_manager.update();
            let plugins_to_reload = self.update_dynamic_plugins(plugins_to_remove);
            if !plugins_to_reload.is_empty() {
                SharedData::flush_resources(&self.shared_data, &self.message_hub);
            }
            self.reload_dynamic_plugins(plugins_to_reload);
        }
        SharedData::flush_resources(&self.shared_data, &self.message_hub);

        can_continue
    }

    pub fn add_static_plugin(&mut self, plugin_holder: PluginHolder) {
        PluginManager::prepare_plugin_holder(&plugin_holder, self);
        self.plugin_manager.add_static_plugin(plugin_holder);
    }

    pub fn add_dynamic_plugin(&mut self, lib_path: &Path) {
        let plugin_data = PluginManager::create_plugin_data(lib_path, self);
        self.plugin_manager.add_dynamic_plugin(plugin_data);
    }

    pub fn remove_static_plugin(&mut self, plugin_id: &PluginId) {
        if let Some(plugin_holder) = self.plugin_manager.remove_static_plugin(plugin_id) {
            PluginManager::release_plugin_holder(plugin_holder, self);
        }
    }

    pub fn remove_dynamic_plugin(&mut self, plugin_id: &PluginId) {
        if let Some(plugin_data) = self.plugin_manager.remove_dynamic_plugin(plugin_id) {
            PluginManager::clear_plugin_data(plugin_data, self);
        }
    }
    fn add_worker(&mut self, name: &str) -> &mut Worker {
        let key = String::from(name);
        let w = self.workers.entry(key).or_insert_with(Worker::default);
        if !w.is_started() {
            w.start(name, &self.job_handler, self.receiver.clone());
        }
        w
    }
    fn get_worker(&mut self, name: &str) -> &mut Worker {
        let key = String::from(name);
        self.workers.get_mut(&key).unwrap()
    }

    pub fn get_job_handler(&self) -> &JobHandlerRw {
        &self.job_handler
    }
    pub fn get_shared_data(&self) -> &SharedDataRc {
        &self.shared_data
    }
    pub fn get_message_hub(&self) -> &MessageHubRc {
        &self.message_hub
    }
    pub fn add_system<S>(&mut self, phase: Phases, system: S)
    where
        S: System + 'static,
    {
        self.scheduler.add_system(phase, system, &self.job_handler);
    }
    pub fn remove_system(&mut self, phase: Phases, system_id: &SystemId) {
        self.scheduler.remove_system(phase, system_id);
    }
    pub fn execute_on_system<S, F>(&mut self, f: F)
    where
        S: System + Sized + 'static,
        F: FnMut(&mut S) + Copy,
    {
        self.scheduler.execute_on_system::<S, F>(f);
    }
}
