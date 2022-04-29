use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use inox_messenger::Listener;
use inox_platform::{InputState, Key, KeyEvent, WindowEvent};
use inox_resources::DeserializeFunction;
use inox_uid::generate_uid_from_string;

use crate::{
    ContextRc, JobHandlerTrait, JobPriority, PluginHolder, PluginId, PluginManager, System,
    SystemEvent, Worker,
};

#[cfg(target_arch = "wasm32")]
const NUM_WORKER_THREADS: usize = 0;
#[cfg(all(not(target_arch = "wasm32")))]
const NUM_WORKER_THREADS: usize = 5;

pub struct App {
    context: ContextRc,
    is_profiling: bool,
    is_enabled: Arc<AtomicBool>,
    listener: Listener,
    plugin_manager: PluginManager,
    workers: HashMap<String, Worker>,
}

impl Default for App {
    fn default() -> Self {
        inox_profiler::create_profiler!();

        let context = ContextRc::default();
        let listener = Listener::new(context.message_hub());

        listener
            .register::<KeyEvent>()
            .register::<WindowEvent>()
            .register::<SystemEvent>();

        Self {
            is_enabled: Arc::new(AtomicBool::new(true)),
            is_profiling: false,
            plugin_manager: PluginManager::default(),
            workers: HashMap::new(),
            context,
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

        self.context.scheduler_mut().uninit();

        let (dynamic_plugins_to_remove, static_plugins_to_remove) = self.plugin_manager.release();
        self.update_dynamic_plugins(dynamic_plugins_to_remove);
        self.remove_static_plugins(static_plugins_to_remove);

        self.listener
            .unregister::<SystemEvent>()
            .unregister::<KeyEvent>()
            .unregister::<WindowEvent>();
    }
}

impl App {
    pub fn start(&mut self) -> &mut Self {
        self.context.global_timer_mut().update();
        self.setup_worker_threads();
        self.context.scheduler_mut().start();
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
        self.context.job_handler().clear_pending_jobs();
    }

    fn update_events(&mut self) {
        inox_profiler::scoped_profile!("app::update_events");

        let mut is_profiling = self.is_profiling;
        let mut is_enabled = self.is_enabled.load(Ordering::SeqCst);

        self.listener
            .process_messages(|e: &KeyEvent| {
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
            })
            .process_messages(|e: &WindowEvent| match e {
                WindowEvent::Show => {
                    is_enabled = true;
                }
                WindowEvent::Hide => {
                    is_enabled = false;
                }
                _ => {}
            });
        self.context
            .shared_data()
            .handle_events(|load_fn: Box<dyn DeserializeFunction>| {
                let shared_data = self.context.shared_data().clone();
                let message_hub = self.context.message_hub().clone();
                let job_name = "Load Event".to_string();
                let load_event_category = generate_uid_from_string("LOAD_EVENT_CATEGORY");
                self.context.job_handler().add_job(
                    &load_event_category,
                    job_name.as_str(),
                    JobPriority::Low,
                    move || {
                        load_fn(&shared_data, &message_hub);
                    },
                );
            });

        //flush messages between frames
        self.context.message_hub().flush();

        self.is_profiling = is_profiling;

        self.update_workers(is_enabled);
    }

    fn update_workers(&mut self, is_enabled: bool) {
        if NUM_WORKER_THREADS == 0 {
            //no workers - need to handle events ourself
            self.context.job_handler().execute_all_jobs();
        }
        if self.is_enabled.load(Ordering::SeqCst) && !is_enabled {
            self.is_enabled.store(is_enabled, Ordering::SeqCst);
            self.stop_worker_threads();
        } else if !self.is_enabled.load(Ordering::SeqCst) && is_enabled {
            self.is_enabled.store(is_enabled, Ordering::SeqCst);
            self.setup_worker_threads();
        }
    }

    pub fn run(&mut self) -> bool {
        inox_profiler::scoped_profile!("app::run_frame");

        self.context.global_timer_mut().update();

        let can_continue = self.context.scheduler_mut().run_once(
            self.is_enabled.load(Ordering::SeqCst),
            self.context.job_handler(),
        );

        self.update_events();

        if !self.is_enabled.load(Ordering::SeqCst) {
            let plugins_to_remove = self.plugin_manager.update();
            let plugins_to_reload = self.update_dynamic_plugins(plugins_to_remove);
            if !plugins_to_reload.is_empty() {
                self.context
                    .shared_data()
                    .flush_resources(self.context.message_hub());
            }
            self.reload_dynamic_plugins(plugins_to_reload);
        }
        self.context
            .shared_data()
            .flush_resources(self.context.message_hub());

        if !can_continue {
            self.is_enabled.store(false, Ordering::SeqCst);
        }

        can_continue
    }

    pub fn add_static_plugin(&mut self, plugin_holder: PluginHolder) -> PluginId {
        PluginManager::load_config_plugin_holder(&plugin_holder, self.context());
        PluginManager::prepare_plugin_holder(&plugin_holder, self.context());
        self.plugin_manager.add_static_plugin(plugin_holder)
    }

    pub fn add_dynamic_plugin(&mut self, lib_path: &Path) -> PluginId {
        let plugin_data = PluginManager::create_plugin_data(lib_path, self.context());
        self.plugin_manager.add_dynamic_plugin(plugin_data)
    }

    pub fn load_config_on_plugin_systems(&mut self, plugin_name: &str) {
        self.execute_on_systems(|s| {
            s.read_config(plugin_name);
        });
    }

    pub fn remove_static_plugin(&mut self, plugin_id: &PluginId) {
        if let Some(plugin_holder) = self.plugin_manager.remove_static_plugin(plugin_id) {
            PluginManager::release_plugin_holder(plugin_holder, self.context());
        }
    }

    pub fn remove_dynamic_plugin(&mut self, plugin_id: &PluginId) {
        if let Some(plugin_data) = self.plugin_manager.remove_dynamic_plugin(plugin_id) {
            PluginManager::clear_plugin_data(plugin_data, self.context());
        }
    }

    fn update_dynamic_plugins(&mut self, plugins_to_remove: Vec<PluginId>) -> Vec<PathBuf> {
        let mut plugins_to_reload = Vec::new();
        for id in plugins_to_remove.iter() {
            if let Some(plugin_data) = self.plugin_manager.remove_dynamic_plugin(id) {
                let lib_path = plugin_data.original_path.clone();
                PluginManager::clear_plugin_data(plugin_data, self.context());
                plugins_to_reload.push(lib_path);
            }
        }
        plugins_to_reload
    }

    fn remove_static_plugins(&mut self, plugins_to_remove: Vec<PluginId>) {
        for id in plugins_to_remove.iter() {
            self.remove_static_plugin(id);
        }
    }

    fn reload_dynamic_plugins(&mut self, plugins_to_reload: Vec<PathBuf>) {
        for lib_path in plugins_to_reload.into_iter() {
            let reloaded_plugin_data =
                PluginManager::create_plugin_data(lib_path.as_path(), self.context());
            self.plugin_manager.add_dynamic_plugin(reloaded_plugin_data);
        }
    }

    fn add_worker(&mut self, name: &str) -> &mut Worker {
        let key = String::from(name);
        let w = self.workers.entry(key).or_insert_with(Worker::default);
        if !w.is_started() {
            w.start(name, &self.is_enabled, self.context.job_handler());
        }
        w
    }
    pub fn context(&self) -> &ContextRc {
        &self.context
    }

    pub fn execute_on_systems<F>(&mut self, f: F)
    where
        F: FnMut(&mut dyn System) + Copy,
    {
        self.context.scheduler_mut().execute_on_systems::<F>(f);
    }
    pub fn execute_on_system<S, F>(&mut self, f: F)
    where
        S: System + Sized + 'static,
        F: FnMut(&mut S) + Copy,
    {
        self.context.scheduler_mut().execute_on_system::<S, F>(f);
    }
}
