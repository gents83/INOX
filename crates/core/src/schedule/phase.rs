use downcast_rs::{impl_downcast, Downcast};

use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::{JobHandlerRw, JobHandlerTrait, JobId, System, SystemId, SystemRunner};

pub trait Phase: Downcast + Send + Sync {
    fn get_name(&self) -> &str;
    fn should_run_when_not_focused(&self) -> bool;
    fn get_jobs_id_to_wait(&self) -> Vec<JobId>;
    fn init(&mut self);
    fn run(&mut self, is_focused: bool, job_handler: &JobHandlerRw) -> bool;
    fn uninit(&mut self);
}
impl_downcast!(Phase);

pub struct PhaseWithSystems {
    name: String,
    systems_runners: HashMap<SystemId, SystemRunner>,
    systems_running: Vec<SystemId>,
    systems_to_add: Vec<SystemId>,
    systems_to_remove: Vec<SystemId>,
    can_continue: Arc<AtomicBool>,
}

impl PhaseWithSystems {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            systems_runners: HashMap::new(),
            systems_running: Vec::new(),
            systems_to_add: Vec::new(),
            systems_to_remove: Vec::new(),
            can_continue: Arc::new(AtomicBool::new(true)),
        }
    }
    pub fn execute_on_systems<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut dyn System),
    {
        self.systems_runners
            .iter_mut()
            .for_each(|(_, system_data)| {
                system_data.call_fn(f);
            });
    }
    pub fn execute_on_system<S, F>(&mut self, f: F)
    where
        S: System + Sized + 'static,
        F: FnMut(&mut S),
    {
        if let Some(system_data) = self.systems_runners.get_mut(&S::system_id()) {
            system_data.execute_on_system(f);
        }
    }
    pub fn add_system_with_dependencies<S>(
        &mut self,
        system: S,
        dependencies: Option<&[SystemId]>,
        job_handler: &JobHandlerRw,
    ) -> &mut Self
    where
        S: System,
    {
        let id = S::system_id();
        self.add_system(system, job_handler);
        if let Some(dependencies) = dependencies {
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
        }
        self
    }

    fn add_system<S>(&mut self, system: S, job_handler: &JobHandlerRw) -> &mut Self
    where
        S: System + 'static,
    {
        let id = S::system_id();
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
        if !self.systems_to_remove.contains(system_id) {
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
        job_handler: &JobHandlerRw,
    ) -> bool {
        inox_profiler::scoped_profile!("phase::execute_systems");
        let mut should_wait = true;
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
                            system_runner.execute_as_job(self.can_continue.clone(), is_focused);
                        } else {
                            system_runner.execute(self.can_continue.clone(), is_focused);
                        }
                    }
                }
            });
            if should_wait {
                if let Some(job) = job_handler.get_job_with_priority(crate::JobPriority::High) {
                    job.execute();
                } else if let Some(job) =
                    job_handler.get_job_with_priority(crate::JobPriority::Medium)
                {
                    job.execute();
                }
            }
        }
        self.can_continue.load(Ordering::Relaxed)
    }

    fn execute(&mut self, is_focused: bool, job_handler: &JobHandlerRw) -> bool {
        #[cfg(target_arch = "wasm32")]
        let execute_in_parallel = false;
        #[cfg(all(not(target_arch = "wasm32")))]
        let execute_in_parallel = self.systems_running.len() > 1;
        self.execute_systems(is_focused, execute_in_parallel, job_handler)
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

    fn run(&mut self, is_focused: bool, job_handler: &JobHandlerRw) -> bool {
        self.remove_pending_systems_from_execution()
            .add_pending_systems_into_execution()
            .execute(is_focused, job_handler)
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
