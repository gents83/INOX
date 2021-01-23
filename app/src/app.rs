use super::scheduler::*;
use super::phase::*;

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

    pub fn create_phase<T: Phase>(&mut self, phase: T) -> &mut Self {
        self.scheduler.create_phase(phase);
        self
    }

    pub fn create_phase_with_systems(&mut self, phase_name: &str) -> &mut Self {
        self.scheduler.create_phase_with_systems(phase_name);
        self
    }

    pub fn get_phase<S: Phase>(&mut self, phase_name:&str) -> &S {
        self.scheduler.get_phase(phase_name)
    }

    pub fn get_phase_mut<S: Phase>(&mut self, phase_name:&str) -> &mut S {
        self.scheduler.get_phase_mut(phase_name)
    }
}