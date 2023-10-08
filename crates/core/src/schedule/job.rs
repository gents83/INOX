use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex, RwLock,
    },
};

use inox_uid::Uid;

use crate::Worker;

#[cfg(target_arch = "wasm32")]
const NUM_WORKER_THREADS: usize = 0;
#[cfg(not(target_arch = "wasm32"))]
const NUM_WORKER_THREADS: usize = 5;

pub type JobId = Uid;
pub const INDEPENDENT_JOB_ID: JobId = inox_uid::generate_static_uid_from_string("IndependentJob");

pub struct Job {
    func: Option<Box<dyn FnOnce() + Send + Sync>>,
    pending_jobs: Arc<AtomicUsize>,
    name: String,
}

unsafe impl Sync for Job {}
unsafe impl Send for Job {}

impl Job {
    pub fn new<F>(name: &str, func: F, pending_jobs: Arc<AtomicUsize>) -> Self
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        pending_jobs.fetch_add(1, Ordering::SeqCst);
        /*
        debug_log(
            "Adding job {:?} - remaining {:?}",
            name,
            pending_jobs.load(Ordering::SeqCst)
        );*/
        Self {
            func: Some(Box::new(func)),
            pending_jobs,
            name: String::from(name),
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn execute(mut self) {
        inox_profiler::scoped_profile!("Job {}", self.name);
        /*
        debug_log(
            "Starting {:?} - remaining {:?}",
            self.name.as_str(),
            self.pending_jobs.load(Ordering::SeqCst)
        );
        */
        let f = self.func.take().unwrap();
        (f)();

        self.pending_jobs.fetch_sub(1, Ordering::SeqCst);
        self.name.clear();
        /*
        debug_log(
            "Ending {:?} - remaining {:?}",
            self.name.as_str(),
            self.pending_jobs.load(Ordering::SeqCst)
        );
        */
    }
}

pub type JobHandlerRw = Arc<RwLock<JobHandler>>;
pub type JobReceiverRw = Arc<Mutex<Receiver<Job>>>;

#[derive(Debug)]
pub enum JobPriority {
    High = 0,
    Medium = 1,
    Low = 2,
    Count = 3,
}
impl From<usize> for JobPriority {
    fn from(value: usize) -> Self {
        match value {
            0 => JobPriority::High,
            1 => JobPriority::Medium,
            2 => JobPriority::Low,
            3 => JobPriority::Count,
            _ => panic!("Invalid job priority"),
        }
    }
}
struct PrioChannel {
    sender: Sender<Job>,
    receiver: JobReceiverRw,
}
impl Default for PrioChannel {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}

#[derive(Default)]
pub struct JobHandler {
    channel: [PrioChannel; JobPriority::Count as usize],
    pending_jobs: HashMap<JobId, Arc<AtomicUsize>>,
    workers: HashMap<String, Worker>,
}

unsafe impl Sync for JobHandler {}
unsafe impl Send for JobHandler {}

impl JobHandler {
    #[inline]
    fn get_pending_jobs_count(&self, job_category: &JobId) -> usize {
        inox_profiler::scoped_profile!("JobHandler::get_pending_jobs_count");
        if let Some(pending_jobs) = self.pending_jobs.get(job_category) {
            pending_jobs.load(Ordering::SeqCst)
        } else {
            0
        }
    }
    #[inline]
    fn get_job_with_priority(&self, job_priority: JobPriority) -> Option<Job> {
        inox_profiler::scoped_profile!("JobReceiver::get_job_with_priority[{:?}]", job_priority);
        self.channel[job_priority as usize].receiver.get_job()
    }
    #[inline]
    fn execute_all_jobs(&self) {
        inox_profiler::scoped_profile!("JobHandler::execute_all_jobs");
        for i in 0..JobPriority::Count as usize {
            while let Some(job) = self.get_job_with_priority(JobPriority::from(i)) {
                job.execute();
            }
        }
    }

    fn add_worker(&mut self, name: &str, can_continue: &Arc<AtomicBool>) -> &mut Worker {
        let key = String::from(name);
        let w = self.workers.entry(key).or_default();
        if !w.is_started() {
            w.start(
                name,
                can_continue,
                self.channel
                    .iter()
                    .map(|channel| channel.receiver.clone())
                    .collect(),
            );
        }
        w
    }

    #[inline]
    fn setup_worker_threads(&mut self, can_continue: &Arc<AtomicBool>) {
        if NUM_WORKER_THREADS > 0 {
            for i in 1..NUM_WORKER_THREADS + 1 {
                self.add_worker(format!("Worker{i}").as_str(), can_continue);
            }
        }
    }

    #[inline]
    fn clear(&mut self) {
        for (_name, w) in self.workers.iter_mut() {
            w.stop();
        }
        self.pending_jobs.iter().for_each(|(_, pending_jobs)| {
            pending_jobs.store(0, Ordering::SeqCst);
        });
        self.pending_jobs.clear();

        self.channel.iter_mut().for_each(|c| {
            while let Some(j) = c.receiver.get_job() {
                drop(j);
            }
        });
    }

    fn add_job<F>(
        &mut self,
        job_category: &JobId,
        job_name: &str,
        job_priority: JobPriority,
        func: F,
    ) where
        F: FnOnce() + Send + Sync + 'static,
    {
        inox_profiler::scoped_profile!("JobHandler::add_job[{}]", job_name);
        let pending_jobs = self
            .pending_jobs
            .entry(*job_category)
            .or_insert_with(|| Arc::new(AtomicUsize::new(0)))
            .clone();
        let job = Job::new(job_name, func, pending_jobs);
        self.channel[job_priority as usize].sender.send(job).ok();
        self.workers.iter().for_each(|(_n, w)| {
            w.wakeup();
        });
    }
}

pub trait JobHandlerTrait {
    fn add_job<F>(&self, job_category: &JobId, job_name: &str, job_priority: JobPriority, func: F)
    where
        F: FnOnce() + Send + Sync + 'static;
    fn get_job_with_priority(&self, job_priority: JobPriority) -> Option<Job>;
    fn has_pending_jobs(&self, job_category: &JobId) -> bool;
    fn update_workers(&self, can_continue: &Arc<AtomicBool>, is_enabled: bool);
    fn start(&self, can_continue: &Arc<AtomicBool>);
    fn stop(&self);
}

impl JobHandlerTrait for JobHandlerRw {
    #[inline]
    fn add_job<F>(&self, job_category: &JobId, job_name: &str, job_priority: JobPriority, func: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.write()
            .unwrap()
            .add_job(job_category, job_name, job_priority, func);
    }

    #[inline]
    fn has_pending_jobs(&self, job_category: &JobId) -> bool {
        self.read().unwrap().get_pending_jobs_count(job_category) > 0
    }
    #[inline]
    fn get_job_with_priority(&self, job_priority: JobPriority) -> Option<Job> {
        self.read().unwrap().get_job_with_priority(job_priority)
    }
    #[inline]
    fn start(&self, can_continue: &Arc<AtomicBool>) {
        self.write().unwrap().setup_worker_threads(can_continue);
    }
    #[inline]
    fn stop(&self) {
        self.write().unwrap().clear();
    }

    fn update_workers(&self, can_continue: &Arc<AtomicBool>, is_enabled: bool) {
        if NUM_WORKER_THREADS == 0 {
            //no workers - need to handle events ourself
            self.read().unwrap().execute_all_jobs();
        }
        if can_continue.load(Ordering::SeqCst) && !is_enabled {
            can_continue.store(is_enabled, Ordering::SeqCst);
            self.stop();
        } else if !can_continue.load(Ordering::SeqCst) && is_enabled {
            can_continue.store(is_enabled, Ordering::SeqCst);
            self.start(can_continue);
        }
    }
}

pub trait JobReceiverTrait {
    fn get_job(&self) -> Option<Job>;
}

impl JobReceiverTrait for JobReceiverRw {
    fn get_job(&self) -> Option<Job> {
        inox_profiler::scoped_profile!("JobReceiver::get_job");
        let mutex = self.lock().unwrap();
        if let Ok(job) = mutex.try_recv() {
            drop(mutex);
            return Some(job);
        }
        None
    }
}
