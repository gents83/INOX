use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{JobReceiverRw, JobReceiverTrait};

#[derive(Default)]
pub struct Worker {
    thread_handle: Option<JoinHandle<bool>>,
}

impl Worker {
    pub fn is_started(&self) -> bool {
        self.thread_handle.is_some()
    }
    pub fn start(
        &mut self,
        name: &str,
        can_continue: &Arc<AtomicBool>,
        receivers: Vec<JobReceiverRw>,
    ) {
        if self.thread_handle.is_none() {
            let builder = thread::Builder::new().name(name.into());
            let can_continue = can_continue.clone();

            let t = builder
                .spawn(move || {
                    inox_profiler::register_profiler_thread!();
                    loop {
                        let mut executed = false;
                        for r in &receivers {
                            if let Some(job) = r.get_job() {
                                job.execute();
                                executed = true;
                                break;
                            }
                        }
                        if !executed {
                            if !can_continue.load(Ordering::SeqCst) {
                                return false;
                            }
                            thread::park_timeout(Duration::from_millis(10));
                        }
                    }
                })
                .unwrap();
            self.thread_handle = Some(t);
        }
    }

    pub fn wakeup(&self) {
        if let Some(t) = &self.thread_handle {
            t.thread().unpark();
        }
    }

    pub fn stop(&mut self) {
        self.wakeup();
        if let Some(t) = self.thread_handle.take() {
            t.join().unwrap();
            self.thread_handle = None;
        }
    }
}
