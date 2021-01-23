use super::scheduler::*;

pub struct App {
    scheduler: Scheduler,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            scheduler: Scheduler::new(),
        }
    }

    pub fn run_once(&mut self) {
        self.scheduler.run_once();
    }

    pub fn run(&mut self) {
        self.scheduler.run();
    }
}