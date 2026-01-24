use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam_channel::{Receiver, Select};

use crate::Job;

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
        receivers: Vec<Receiver<Job>>,
    ) {
        if self.thread_handle.is_none() {
            let builder = thread::Builder::new().name(name.into());
            let can_continue = can_continue.clone();

            let t = builder
                .spawn(move || {
                    inox_profiler::register_profiler_thread!();
                    loop {
                        let mut executed = false;
                        // Try to execute jobs based on priority (order in receivers)
                        for r in &receivers {
                            if let Ok(job) = r.try_recv() {
                                job.execute();
                                executed = true;
                                break;
                            }
                        }

                        if !executed {
                            if !can_continue.load(Ordering::SeqCst) {
                                return false;
                            }

                            let mut sel = Select::new();
                            for r in &receivers {
                                sel.recv(r);
                            }

                            // Wait for a job or timeout to check can_continue
                            if let Ok(oper) = sel.select_timeout(Duration::from_millis(100)) {
                                let index = oper.index();
                                if let Ok(job) = oper.recv(&receivers[index]) {
                                    job.execute();
                                }
                            }
                        }
                    }
                })
                .unwrap();
            self.thread_handle = Some(t);
        }
    }

    pub fn wakeup(&self) {
        // No-op
    }

    pub fn stop(&mut self) {
        if let Some(t) = self.thread_handle.take() {
            t.join().unwrap();
            self.thread_handle = None;
        }
    }
}
