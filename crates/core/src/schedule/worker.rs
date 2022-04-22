use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use crate::{JobHandlerRw, JobHandlerTrait, JobReceiverTrait};

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
        job_handler: &JobHandlerRw,
    ) {
        if self.thread_handle.is_none() {
            let receivers = job_handler.receivers();
            let builder = thread::Builder::new().name(name.into());
            let can_continue = can_continue.clone();

            let t = builder
                .spawn(move || {
                    inox_profiler::register_thread!();
                    loop {
                        let mut i = 0;
                        while i < receivers.len() {
                            if let Some(job) = receivers[i].get_job() {
                                job.execute();
                                //force exit from loop
                                i = receivers.len();
                            }
                            i += 1;
                        }
                        if !can_continue.load(Ordering::SeqCst) {
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

            t.join().unwrap();

            self.thread_handle = None;
        }
    }
}
