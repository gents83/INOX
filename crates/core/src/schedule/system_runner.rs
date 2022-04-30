use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc, RwLock,
    },
};

use crate::{JobHandlerRw, JobHandlerTrait, JobPriority, System, SystemId, SystemRw};

const STATE_READY: u8 = 0;
const STATE_WAITING: u8 = 1;
const STATE_RUNNING: u8 = 2;
const STATE_EXECUTED: u8 = 3;

pub struct SystemRunner {
    system_id: SystemId,
    name: String,
    system: SystemRw,
    dependencies: HashMap<SystemId, Arc<AtomicU8>>,
    state: Arc<AtomicU8>,
    job_handler: JobHandlerRw,
}

impl SystemRunner {
    pub fn new<S>(system: S, job_handler: JobHandlerRw) -> Self
    where
        S: System + 'static,
    {
        Self {
            system_id: S::system_id(),
            name: system.name().to_string(),
            system: Arc::new(RwLock::new(Box::new(system))),
            dependencies: HashMap::new(),
            state: Arc::new(AtomicU8::new(STATE_READY)),
            job_handler,
        }
    }
    pub fn init(&mut self) {
        self.system.write().unwrap().init();
    }
    pub fn should_run_when_not_focused(&self) -> bool {
        self.system.read().unwrap().should_run_when_not_focused()
    }
    pub fn uninit(&mut self) {
        self.system.write().unwrap().uninit();
    }
    pub fn state(&self) -> Arc<AtomicU8> {
        self.state.clone()
    }

    pub fn add_dependencies(&mut self, dependencies: HashMap<SystemId, Arc<AtomicU8>>) {
        for (id, state) in dependencies {
            self.dependencies.insert(id, state);
        }
    }

    pub fn call_fn<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut dyn System),
    {
        let mut system = self.system.write().unwrap();
        f(system.as_mut());
    }

    pub fn execute_on_system<S, F>(&mut self, mut f: F)
    where
        S: System + Sized + 'static,
        F: FnMut(&mut S),
    {
        if let Some(system) = self.system.write().unwrap().as_mut().downcast_mut() {
            f(system);
        }
    }

    pub fn start(&mut self) {
        debug_assert!(
            self.state.load(Ordering::SeqCst) == STATE_READY
                || self.state.load(Ordering::SeqCst) == STATE_EXECUTED
        );
        self.state.store(STATE_WAITING, Ordering::SeqCst);
    }

    pub fn is_waiting_dependencies(&mut self) -> bool {
        let mut can_start = true;
        self.dependencies
            .iter()
            .for_each(|(_, dependency_is_running)| {
                can_start &= dependency_is_running.load(Ordering::SeqCst) == STATE_EXECUTED;
            });
        !can_start
    }

    pub fn execute(&mut self, can_continue: Arc<AtomicBool>, is_focused: bool) {
        if self.is_executed() || self.is_running() {
            return;
        }
        let should_run_when_not_focused = self.should_run_when_not_focused();
        if is_focused || should_run_when_not_focused {
            self.state.store(STATE_RUNNING, Ordering::SeqCst);
            let result = can_continue.load(Ordering::SeqCst) && self.system.write().unwrap().run();
            can_continue.store(result, Ordering::SeqCst);
        }
        self.state.store(STATE_EXECUTED, Ordering::SeqCst);
    }

    pub fn execute_as_job(&mut self, can_continue: Arc<AtomicBool>, is_focused: bool) {
        if self.is_executed() || self.is_running() {
            return;
        }
        let should_run_when_not_focused = self.should_run_when_not_focused();
        if is_focused || should_run_when_not_focused {
            self.state.store(STATE_RUNNING, Ordering::SeqCst);
            let state = self.state.clone();
            let system = self.system.clone();
            self.job_handler.add_job(
                &self.system_id,
                format!("execute_system[{}]", self.name).as_str(),
                JobPriority::High,
                move || {
                    let result =
                        can_continue.load(Ordering::SeqCst) && system.write().unwrap().run();
                    can_continue.store(result, Ordering::SeqCst);

                    state.store(STATE_EXECUTED, Ordering::SeqCst);
                },
            );
        } else {
            self.state.store(STATE_EXECUTED, Ordering::SeqCst);
        }
    }

    pub fn is_ready(&self) -> bool {
        self.state.load(Ordering::SeqCst) == STATE_READY
    }
    pub fn is_waiting(&self) -> bool {
        self.state.load(Ordering::SeqCst) == STATE_WAITING
    }
    pub fn is_executed(&self) -> bool {
        self.state.load(Ordering::SeqCst) == STATE_EXECUTED
    }
    pub fn is_running(&self) -> bool {
        self.state.load(Ordering::SeqCst) == STATE_RUNNING
    }
}
