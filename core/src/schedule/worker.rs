use std::{
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex, RwLock,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crate::{Job, Phase, Scheduler};

const MAX_WORKER_DURATION: Duration = Duration::from_millis(2);

pub struct Worker {
    scheduler: Arc<RwLock<Scheduler>>,
    thread_handle: Option<JoinHandle<bool>>,
}

impl Default for Worker {
    fn default() -> Self {
        Self {
            scheduler: Arc::new(RwLock::new(Scheduler::new())),
            thread_handle: None,
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.scheduler.write().unwrap().uninit();
    }
}

impl Worker {
    pub fn is_started(&self) -> bool {
        self.thread_handle.is_some()
    }
    pub fn start(
        &mut self,
        name: &'static str,
        sender: Arc<Mutex<Sender<Job>>>,
        receiver: Arc<Mutex<Receiver<Job>>>,
    ) {
        if self.thread_handle.is_none() {
            println!("Starting thread {}", name);
            let builder = thread::Builder::new().name(name.into());
            let scheduler = Arc::clone(&self.scheduler);
            let t = builder
                .spawn(move || {
                    nrg_profiler::register_thread_into_profiler_with_name!(name);
                    let mut last_update_time = Instant::now();
                    loop {
                        let mut instant = Instant::now();
                        let can_continue = scheduler.write().unwrap().run_once(sender.clone());
                        while instant - last_update_time < MAX_WORKER_DURATION {
                            if let Ok(job) = receiver.lock().unwrap().try_recv() {
                                job.execute();
                            }
                            instant = Instant::now();
                        }
                        last_update_time = instant;
                        if !can_continue {
                            return false;
                        }
                    }
                })
                .unwrap();
            self.thread_handle = Some(t);
        }
    }

    pub fn stop(&mut self) {
        if self.thread_handle.is_some() {
            let t = self.thread_handle.take().unwrap();
            println!("Stopping thread {}", t.thread().name().unwrap_or("no_name"));

            self.scheduler.write().unwrap().cancel();
            t.join().unwrap();

            self.thread_handle = None;
        }
    }

    pub fn create_phase<T: Phase>(&mut self, phase: T) -> &mut Self {
        self.scheduler.write().unwrap().create_phase(phase);
        self
    }

    pub fn destroy_phase(&mut self, phase_name: &str) -> &mut Self {
        self.scheduler.write().unwrap().destroy_phase(phase_name);
        self
    }
}
