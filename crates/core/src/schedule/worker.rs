use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use crate::Job;

#[derive(Default)]
pub struct Worker {
    thread_handle: Option<JoinHandle<bool>>,
}

impl Worker {
    pub fn is_started(&self) -> bool {
        self.thread_handle.is_some()
    }
    pub fn get_job(receiver: &Arc<Mutex<Receiver<Job>>>) -> Option<Job> {
        let recv = receiver.lock().unwrap();
        if let Ok(job) = recv.try_recv() {
            return Some(job);
        }
        None
    }

    pub fn start(
        &mut self,
        name: &str,
        can_continue: &Arc<AtomicBool>,
        job_receiver: Arc<Mutex<Receiver<Job>>>,
    ) {
        if self.thread_handle.is_none() {
            let builder = thread::Builder::new().name(name.into());
            let can_continue = can_continue.clone();

            let t = builder
                .spawn(move || {
                    inox_profiler::register_thread!();
                    loop {
                        while let Some(job) = Worker::get_job(&job_receiver) {
                            job.execute();
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
