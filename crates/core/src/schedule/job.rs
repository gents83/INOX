use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex, RwLock,
    },
};

use inox_uid::Uid;

pub type JobId = Uid;
pub const INDEPENDENT_JOB_ID: JobId = inox_uid::generate_static_uid_from_string("IndependentJob");

pub struct Job {
    func: Box<dyn FnOnce() + Send + Sync>,
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
            func: Box::new(func),
            pending_jobs,
            name: String::from(name),
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn execute(self) {
        inox_profiler::scoped_profile!("{}", self.name);
        /*
        debug_log(
            "Starting {:?} - remaining {:?}",
            self.name.as_str(),
            self.pending_jobs.load(Ordering::SeqCst)
        );
        */

        (self.func)();

        self.pending_jobs.fetch_sub(1, Ordering::SeqCst);
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
}

unsafe impl Sync for JobHandler {}
unsafe impl Send for JobHandler {}

pub trait JobHandlerTrait {
    fn add_job<F>(&self, job_category: &JobId, job_name: &str, job_priority: JobPriority, func: F)
    where
        F: FnOnce() + Send + Sync + 'static;
    fn get_job_with_priority(&self, job_priority: JobPriority) -> Option<Job>;
    fn has_pending_jobs(&self, job_category: &JobId) -> bool;
    fn get_pending_jobs_count(&self, job_category: &JobId) -> usize;
    fn execute_all_jobs(&self);
    fn clear_pending_jobs(&self);
    fn receivers(&self) -> Vec<JobReceiverRw>;
}

impl JobHandlerTrait for JobHandlerRw {
    #[inline]
    fn add_job<F>(&self, job_category: &JobId, job_name: &str, job_priority: JobPriority, func: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        let pending_jobs = self
            .write()
            .unwrap()
            .pending_jobs
            .entry(*job_category)
            .or_insert_with(|| Arc::new(AtomicUsize::new(0)))
            .clone();
        let job = Job::new(job_name, func, pending_jobs);
        self.write().unwrap().channel[job_priority as usize]
            .sender
            .send(job)
            .ok();
    }

    #[inline]
    fn has_pending_jobs(&self, job_category: &JobId) -> bool {
        self.get_pending_jobs_count(job_category) > 0
    }

    #[inline]
    fn get_pending_jobs_count(&self, job_category: &JobId) -> usize {
        if let Some(pending_jobs) = self.read().unwrap().pending_jobs.get(job_category) {
            pending_jobs.load(Ordering::SeqCst)
        } else {
            0
        }
    }
    fn get_job_with_priority(&self, job_priority: JobPriority) -> Option<Job> {
        let handler = self.read().unwrap();
        handler.channel[job_priority as usize].receiver.get_job()
    }
    #[inline]
    fn execute_all_jobs(&self) {
        for i in 0..JobPriority::Count as usize {
            while let Some(job) = self.get_job_with_priority(JobPriority::from(i)) {
                job.execute();
            }
        }
    }

    #[inline]
    fn clear_pending_jobs(&self) {
        self.write()
            .unwrap()
            .pending_jobs
            .iter()
            .for_each(|(_, pending_jobs)| {
                pending_jobs.store(0, Ordering::SeqCst);
            });
        self.write().unwrap().pending_jobs.clear();
    }

    #[inline]
    fn receivers(&self) -> Vec<JobReceiverRw> {
        let handler = self.read().unwrap();
        handler
            .channel
            .iter()
            .map(|channel| channel.receiver.clone())
            .collect()
    }
}

pub trait JobReceiverTrait {
    fn get_job(&self) -> Option<Job>;
}

impl JobReceiverTrait for JobReceiverRw {
    fn get_job(&self) -> Option<Job> {
        let mutex = self.lock().unwrap();
        if let Ok(job) = mutex.try_recv() {
            drop(mutex);
            return Some(job);
        }
        None
    }
}
