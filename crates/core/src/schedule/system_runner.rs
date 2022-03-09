use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};

use crate::{JobHandlerRw, System, SystemId, SystemRw};
pub struct SystemRunner {
    system_id: SystemId,
    system: SystemRw,
    dependencies: Vec<SystemId>,
    is_running: Arc<AtomicBool>,
    job_handler: JobHandlerRw,
}

impl SystemRunner {
    pub fn new<S>(system: S, job_handler: JobHandlerRw) -> Self
    where
        S: System + 'static,
    {
        Self {
            system_id: S::id(),
            system: Arc::new(RwLock::new(Box::new(system))),
            dependencies: Vec::new(),
            is_running: Arc::new(AtomicBool::new(false)),
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

    pub fn execute_on_system<S, F>(&mut self, mut f: F)
    where
        S: System + Sized + 'static,
        F: FnMut(&mut S),
    {
        if let Some(system) = self.system.write().unwrap().as_mut().downcast_mut() {
            f(system);
        }
    }

    pub fn execute(&mut self, can_continue: Arc<AtomicBool>, is_focused: bool) {
        self.is_running.store(true, Ordering::SeqCst);

        if is_focused || self.should_run_when_not_focused() {
            let result = can_continue.load(Ordering::SeqCst) && self.system.write().unwrap().run();
            can_continue.store(result, Ordering::SeqCst);
        }
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub fn execute_as_job(&mut self, can_continue: Arc<AtomicBool>, is_focused: bool) {
        self.is_running.store(true, Ordering::SeqCst);

        if is_focused || self.should_run_when_not_focused() {
            let is_running = self.is_running.clone();
            let system = self.system.clone();
            self.job_handler.write().unwrap().add_job(
                &self.system_id,
                format!("execute_system[{}]", self.system.read().unwrap().get_name()).as_str(),
                move || {
                    let result =
                        can_continue.load(Ordering::SeqCst) && system.write().unwrap().run();
                    can_continue.store(result, Ordering::SeqCst);

                    is_running.store(false, Ordering::SeqCst);
                },
            );
        } else {
            self.is_running.store(false, Ordering::SeqCst);
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}
