use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

pub struct Job {
    func: Box<dyn FnOnce() + Send + Sync>,
    wait_count: Option<Arc<AtomicUsize>>,
    name: String,
}

unsafe impl Sync for Job {}
unsafe impl Send for Job {}

impl Job {
    pub fn new<F>(name: &str, func: F) -> Self
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        Self {
            func: Box::new(func),
            wait_count: None,
            name: String::from(name),
        }
    }

    pub fn set_wait_count(&mut self, wait_count: Arc<AtomicUsize>) {
        self.wait_count = Some(wait_count);
    }

    pub fn execute(mut self) {
        nrg_profiler::scoped_profile!(self.name.as_str());

        (self.func)();

        if let Some(wait_count) = self.wait_count {
            wait_count.fetch_sub(1, Ordering::SeqCst);
        }
        self.wait_count = None;
    }
}
