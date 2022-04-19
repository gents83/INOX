use crate::{
    JobHandlerRw, JobReceiverRw, Phase, PhaseWithSystems, Phases, System, SystemId, Worker,
};
use std::collections::HashMap;

pub struct Scheduler {
    is_running: bool,
    is_started: bool,
    phases: HashMap<Phases, PhaseWithSystems>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Sync for Scheduler {}
unsafe impl Send for Scheduler {}

impl Scheduler {
    pub fn new() -> Self {
        let mut phases = HashMap::new();
        for p in Phases::iterator() {
            phases.insert(p, PhaseWithSystems::new(format!("{:?}", p).as_str()));
        }
        Self {
            is_running: true,
            is_started: false,
            phases,
        }
    }

    pub fn start(&mut self) {
        self.is_started = true;
    }

    pub fn resume(&mut self) {
        self.is_running = true;
    }

    pub fn cancel(&mut self) {
        self.is_running = false;
    }

    pub fn uninit(&mut self) {
        self.cancel();
        for p in Phases::iterator() {
            if let Some(phase) = self.phases.get_mut(&p) {
                phase.uninit();
            }
        }
    }

    pub fn run_once(
        &mut self,
        is_focused: bool,
        job_handler: &JobHandlerRw,
        job_receiver: &JobReceiverRw,
    ) -> bool {
        if !self.is_started {
            return self.is_running;
        }
        inox_profiler::scoped_profile!("scheduler::run_once");
        let mut can_continue = self.is_running;
        for p in Phases::iterator() {
            if let Some(phase) = self.phases.get_mut(&p) {
                let ok = if is_focused || phase.should_run_when_not_focused() {
                    inox_profiler::scoped_profile!("{}[{:?}]", "scheduler::run_phase", p);
                    let ok = phase.run(is_focused, job_receiver);
                    {
                        inox_profiler::scoped_profile!("{}[{:?}]", "scheduler::wait_jobs", p);
                        let jobs_id_to_wait = phase.get_jobs_id_to_wait();
                        let mut should_wait = true;
                        while should_wait {
                            thread::yield_now();
                            should_wait = false;
                            jobs_id_to_wait.iter().for_each(|job_id| {
                                should_wait |= job_handler.read().unwrap().has_pending_jobs(job_id);
                            });
                            if should_wait {
                                if let Some(job) = Worker::get_job(job_receiver) {
                                    job.execute();
                                }
                            }
                        }
                    }
                    ok
                } else {
                    true
                };
                can_continue &= ok;
            }
        }
        can_continue
    }

    pub fn add_system<S>(&mut self, phase: Phases, system: S, job_handler: &JobHandlerRw)
    where
        S: System + 'static,
    {
        if let Some(phase) = self.phases.get_mut(&phase) {
            phase.add_system(system, job_handler);
        }
    }
    pub fn add_system_with_dependencies<S>(
        &mut self,
        phase: Phases,
        system: S,
        dependencies: &[SystemId],
        job_handler: &JobHandlerRw,
    ) where
        S: System + 'static,
    {
        if let Some(phase) = self.phases.get_mut(&phase) {
            phase.add_system_with_dependencies(system, dependencies, job_handler);
        }
    }
    pub fn remove_system(&mut self, phase: Phases, system_id: &SystemId) {
        if let Some(phase) = self.phases.get_mut(&phase) {
            phase.remove_system(system_id);
        }
    }
    pub fn execute_on_systems<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut dyn System),
    {
        self.phases.iter_mut().for_each(|(_, phase)| {
            phase.execute_on_systems::<F>(&mut f);
        });
    }
    pub fn execute_on_system<S, F>(&mut self, f: F)
    where
        S: System + Sized + 'static,
        F: FnMut(&mut S) + Copy,
    {
        self.phases.iter_mut().for_each(|(_, phase)| {
            phase.execute_on_system::<S, F>(f);
        });
    }
}
