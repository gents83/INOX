use downcast_rs::{impl_downcast, Downcast};

use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::{JobHandlerRw, JobId, JobReceiverRw, System, SystemId, SystemRunner, Worker};

pub trait Phase: Downcast + Send + Sync {
    fn get_name(&self) -> &str;
    fn should_run_when_not_focused(&self) -> bool;
    fn get_jobs_id_to_wait(&self) -> Vec<JobId>;
    fn init(&mut self);
    fn run(&mut self, is_focused: bool, job_receiver: &JobReceiverRw) -> bool;
    fn uninit(&mut self);
}
impl_downcast!(Phase);

pub struct PhaseWithSystems {
    name: String,
    systems_runners: HashMap<SystemId, SystemRunner>,
    systems_running: Vec<SystemId>,
    systems_to_add: Vec<SystemId>,
    systems_to_remove: Vec<SystemId>,
}

impl PhaseWithSystems {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            systems_runners: HashMap::new(),
            systems_running: Vec::new(),
            systems_to_add: Vec::new(),
            systems_to_remove: Vec::new(),
        }
    }

    pub fn execute_on_system<S, F>(&mut self, f: F)
    where
        S: System + Sized + 'static,
        F: FnMut(&mut S),
    {
        if let Some(system_data) = self.systems_runners.get_mut(&S::id()) {
            system_data.execute_on_system(f);
        }
    }
    pub fn add_system_with_dependencies<S: System + 'static>(
        &mut self,
        system: S,
        dependencies: &[SystemId],
        job_handler: &JobHandlerRw,
    ) -> &mut Self {
        let id = S::id();
        self.add_system(system, job_handler);
        let mut dependencies_states = HashMap::new();
        dependencies.iter().for_each(|system_id| {
            if let Some(system_data) = self.systems_runners.get_mut(system_id) {
                dependencies_states.insert(*system_id, system_data.state());
            }
        });
        self.systems_runners
            .get_mut(&id)
            .unwrap()
            .add_dependencies(dependencies_states);
        self
    }

    pub fn add_system<S: System + 'static>(
        &mut self,
        system: S,
        job_handler: &JobHandlerRw,
    ) -> &mut Self {
        let id = S::id();
        if let Entry::Vacant(e) = self.systems_runners.entry(id) {
            self.systems_to_add.push(id);
            e.insert(SystemRunner::new(system, job_handler.clone()));
        } else {
            eprintln!(
                "Trying to add twice a System with id {:?} in this Phase",
                id,
            );
        }
        self
    }

    pub fn remove_system(&mut self, system_id: &SystemId) -> &mut Self {
        if !self.systems_runners.contains_key(system_id) {
            eprintln!(
                "Trying to remove a System with id {:?} in this Phase",
                system_id,
            );
        } else if !self.systems_to_remove.contains(system_id) {
            self.systems_to_remove.push(*system_id);
        }
        self
    }

    fn remove_all_systems(&mut self) -> &mut Self {
        for id in self.systems_running.iter() {
            if !self.systems_to_remove.contains(id) {
                self.systems_to_remove.push(*id);
            }
        }
        self
    }

    fn execute_systems(
        &mut self,
        is_focused: bool,
        execute_in_parallel: bool,
        job_receiver: &JobReceiverRw,
    ) -> bool {
        inox_profiler::scoped_profile!("phase::execute_systems");
        let mut should_wait = true;
        let can_continue = Arc::new(AtomicBool::new(true));
        self.systems_running.iter().for_each(|id| {
            if let Some(system_runner) = self.systems_runners.get_mut(id) {
                system_runner.start();
            }
        });
        while should_wait {
            should_wait = false;
            self.systems_running.iter().for_each(|id| {
                if let Some(system_runner) = self.systems_runners.get_mut(id) {
                    if system_runner.is_running()
                        || (system_runner.is_waiting() && system_runner.is_waiting_dependencies())
                    {
                        should_wait = true;
                    } else if !system_runner.is_executed() {
                        should_wait = true;
                        if execute_in_parallel {
                            system_runner.execute_as_job(can_continue.clone(), is_focused);
                        } else {
                            system_runner.execute(can_continue.clone(), is_focused);
                        }
                    }
                }
            });
            if should_wait {
                if let Some(job) = Worker::get_job(job_receiver) {
                    job.execute();
                }
                thread::yield_now();
            }
        }
        can_continue.load(Ordering::Relaxed)
    }

    fn execute(&mut self, is_focused: bool, job_receiver: &JobReceiverRw) -> bool {
        #[cfg(target_arch = "wasm32")]
        let execute_in_parallel = false;
        #[cfg(all(not(target_arch = "wasm32")))]
        let execute_in_parallel = self.systems_running.len() > 1;
        self.execute_systems(is_focused, execute_in_parallel, job_receiver)
    }

    fn remove_pending_systems_from_execution(&mut self) -> &mut Self {
        for id in self.systems_to_remove.iter() {
            if let Some(index) = self.systems_running.iter().position(|s| s == id) {
                self.systems_running.remove(index);
            }
            if let Some(mut system_runner) = self.systems_runners.remove(id) {
                system_runner.uninit();
            }
        }
        self.systems_to_remove.clear();
        self
    }

    fn add_pending_systems_into_execution(&mut self) -> &mut Self {
        for id in self.systems_to_add.iter() {
            if let Some(system_runner) = self.systems_runners.get_mut(id) {
                system_runner.init();
            }
        }
        self.systems_running.append(&mut self.systems_to_add);
        self
    }
}

impl Phase for PhaseWithSystems {
    fn get_name(&self) -> &str {
        &self.name
    }
    fn should_run_when_not_focused(&self) -> bool {
        for id in self.systems_running.iter() {
            if let Some(system_runner) = self.systems_runners.get(id) {
                if system_runner.should_run_when_not_focused() {
                    return true;
                }
            }
        }
        false
    }

    fn init(&mut self) {
        self.add_pending_systems_into_execution();
    }

    fn run(&mut self, is_focused: bool, job_receiver: &JobReceiverRw) -> bool {
        self.remove_pending_systems_from_execution()
            .add_pending_systems_into_execution()
            .execute(is_focused, job_receiver)
    }

    fn uninit(&mut self) {
        self.remove_all_systems()
            .remove_pending_systems_from_execution();

        self.systems_running.clear();
    }

    fn get_jobs_id_to_wait(&self) -> Vec<JobId> {
        self.systems_running.clone()
    }
}

unsafe impl Send for PhaseWithSystems {}
unsafe impl Sync for PhaseWithSystems {}
