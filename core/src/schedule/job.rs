pub struct Job {
    func: Box<dyn FnOnce() + Send + Sync>,
}

impl Job {
    pub fn new<F>(func: F) -> Self
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        Self {
            func: Box::new(func),
        }
    }

    pub fn execute(self) {
        (self.func)()
    }
}
