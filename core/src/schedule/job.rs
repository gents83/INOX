use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::Sender,
        Arc, RwLock,
    },
};

use nrg_serialize::Uid;

pub type JobId = Uid;

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
        nrg_profiler::scoped_profile!(self.name.as_str());
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

pub struct JobHandler {
    sender: Sender<Job>,
    pending_jobs: HashMap<JobId, Arc<AtomicUsize>>,
}

unsafe impl Sync for JobHandler {}
unsafe impl Send for JobHandler {}

impl JobHandler {
    #[inline]
    pub fn new(sender: Sender<Job>) -> Arc<RwLock<JobHandler>> {
        Arc::new(RwLock::new(JobHandler {
            sender,
            pending_jobs: HashMap::new(),
        }))
    }
    #[inline]
    pub fn add_job<F>(&mut self, job_category: &JobId, job_name: &str, func: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        let pending_jobs = self
            .pending_jobs
            .entry(*job_category)
            .or_insert_with(|| Arc::new(AtomicUsize::new(0)))
            .clone();
        let job = Job::new(job_name, func, pending_jobs);
        self.sender.send(job).ok();
    }

    #[inline]
    pub fn has_pending_jobs(&self, job_category: &JobId) -> bool {
        self.get_pending_jobs_count(job_category) > 0
    }

    #[inline]
    pub fn get_pending_jobs_count(&self, job_category: &JobId) -> usize {
        if let Some(pending_jobs) = self.pending_jobs.get(job_category) {
            pending_jobs.load(Ordering::SeqCst)
        } else {
            0
        }
    }

    #[inline]
    pub fn clear_pending_jobs(&mut self) {
        self.pending_jobs.iter().for_each(|(_, pending_jobs)| {
            pending_jobs.store(0, Ordering::SeqCst);
        });
        self.pending_jobs.clear();
    }
}
