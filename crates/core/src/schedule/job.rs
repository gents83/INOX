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
const LOW_PRIORITY_THREAD_RATIO: f32 = 0.5;

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
        let f = self.func.take().unwrap();
        (f)();

        self.pending_jobs.fetch_sub(1, Ordering::SeqCst);
        self.name.clear();
    }
}

pub type JobHandlerRw = Arc<RwLock<JobHandler>>;
pub type JobReceiverRw = Arc<Mutex<Receiver<Job>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pending_jobs: RwLock<HashMap<JobId, Arc<AtomicUsize>>>,
    workers: HashMap<String, Worker>,
}

unsafe impl Sync for JobHandler {}
unsafe impl Send for JobHandler {}

impl JobHandler {
    #[inline]
    fn get_pending_jobs_count(&self, job_category: &JobId) -> usize {
        inox_profiler::scoped_profile!("JobHandler::get_pending_jobs_count");
        if let Ok(pending_jobs) = self.pending_jobs.read() {
            if let Some(pending_jobs) = pending_jobs.get(job_category) {
                return pending_jobs.load(Ordering::SeqCst);
            }
        }
        0
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

    fn add_worker(
        &mut self,
        name: &str,
        can_continue: &Arc<AtomicBool>,
        receivers: Vec<JobReceiverRw>,
    ) -> &mut Worker {
        let key = String::from(name);
        let w = self.workers.entry(key).or_default();
        if !w.is_started() {
            w.start(name, can_continue, receivers);
        }
        w
    }

    #[inline]
    fn setup_worker_threads(&mut self, can_continue: &Arc<AtomicBool>) {
        if NUM_WORKER_THREADS > 0 {
            // High priority jobs are mandatory and should be executed as fast as possible
            // Low priority jobs are non-mandatory and should not block the frame
            // We can set a ratio of threads that can execute Low priority jobs
            let num_low_priority_workers = (NUM_WORKER_THREADS as f32 * LOW_PRIORITY_THREAD_RATIO).ceil() as usize;
            let num_low_priority_workers = num_low_priority_workers.max(1);

            for i in 0..NUM_WORKER_THREADS {
                let mut receivers = vec![
                    self.channel[JobPriority::High as usize].receiver.clone(),
                    self.channel[JobPriority::Medium as usize].receiver.clone(),
                ];
                if i < num_low_priority_workers {
                    receivers.push(self.channel[JobPriority::Low as usize].receiver.clone());
                }

                self.add_worker(
                    format!("Worker{i}").as_str(),
                    can_continue,
                    receivers,
                );
            }
        }
    }

    #[inline]
    fn clear(&mut self) {
        for (_name, w) in self.workers.iter_mut() {
            w.stop();
        }
        if let Ok(mut pending_jobs) = self.pending_jobs.write() {
            pending_jobs.iter().for_each(|entry| {
                entry.1.store(0, Ordering::SeqCst);
            });
            pending_jobs.clear();
        }

        self.channel.iter_mut().for_each(|c| {
            while let Some(j) = c.receiver.get_job() {
                drop(j);
            }
        });
    }

    fn add_job<F>(
        &self,
        job_category: &JobId,
        job_name: &str,
        job_priority: JobPriority,
        func: F,
    ) where
        F: FnOnce() + Send + Sync + 'static,
    {
        inox_profiler::scoped_profile!("JobHandler::add_job[{}]", job_name);
        let pending_jobs = {
            let read_lock = self.pending_jobs.read().unwrap();
            read_lock.get(job_category).cloned()
        };
        let pending_jobs = pending_jobs.unwrap_or_else(|| {
            let mut write_lock = self.pending_jobs.write().unwrap();
            write_lock
                .entry(*job_category)
                .or_insert_with(|| Arc::new(AtomicUsize::new(0)))
                .clone()
        });

        let job = Job::new(job_name, func, pending_jobs);
        self.channel[job_priority as usize].sender.send(job).ok();
        // Wake up all workers as we don't know which one is sleeping on this priority
        // Using unpark is cheap
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
        self.read()
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
