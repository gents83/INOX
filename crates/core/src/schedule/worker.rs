use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
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
                    let receivers_count = receivers.len() as i32;
                    let mut i;
                    loop {
                        i = 0i32;
                        while i >= 0 && i < receivers_count {
                            if let Some(job) = receivers[i as usize].get_job() {
                                job.execute();
                                //force exit from loop
                                i = -1;
                            } else {
                                i += 1;
                            }
                        }
                        if !can_continue.load(Ordering::SeqCst) {
                            return false;
                        }
                        if i >= 0 {
                            std::thread::park();
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
        if self.thread_handle.is_some() {
            let t = self.thread_handle.take().unwrap();
            t.join().unwrap();

            self.thread_handle = None;
        }
    }
}
