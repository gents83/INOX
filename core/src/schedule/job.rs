use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc::Sender,
    Arc, RwLock,
};
pub struct Job {
    func: Box<dyn FnOnce() + Send + Sync>,
    wait_count: Arc<AtomicUsize>,
    name: String,
}

unsafe impl Sync for Job {}
unsafe impl Send for Job {}

impl Job {
    pub fn new<F>(name: &str, func: F, wait_count: Arc<AtomicUsize>) -> Self
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        Self {
            func: Box::new(func),
            wait_count,
            name: String::from(name),
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn execute(self) {
        nrg_profiler::scoped_profile!(self.name.as_str());

        (self.func)();

        self.wait_count.fetch_sub(1, Ordering::SeqCst);
    }
}

pub type JobHandlerRw = Arc<RwLock<JobHandler>>;

pub struct JobHandler {
    sender: Sender<Job>,
    pending_jobs: Arc<AtomicUsize>,
}

unsafe impl Sync for JobHandler {}
unsafe impl Send for JobHandler {}

impl JobHandler {
    #[inline]
    pub fn new(sender: Sender<Job>) -> Arc<RwLock<JobHandler>> {
        Arc::new(RwLock::new(JobHandler {
            sender,
            pending_jobs: Arc::new(AtomicUsize::new(0)),
        }))
    }
    #[inline]
    pub fn add_job<F>(&mut self, job_name: &str, func: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.pending_jobs.fetch_add(1, Ordering::SeqCst);

        let job = Job::new(job_name, func, self.pending_jobs.clone());
        self.sender.send(job).ok();
    }

    #[inline]
    pub fn has_pending_jobs(&self) -> bool {
        self.get_pending_jobs_count() > 0
    }

    #[inline]
    pub fn get_pending_jobs_count(&self) -> usize {
        self.pending_jobs.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn clear_pending_jobs(&self) {
        if self.has_pending_jobs() {
            self.pending_jobs.store(0, Ordering::SeqCst);
        }
    }
}
