use downcast_rs::{impl_downcast, Downcast};

use std::collections::{hash_map::Entry, HashMap};

use crate::{JobId, System, SystemBoxed, SystemId};

pub trait Phase: Downcast + Send + Sync {
    fn get_name(&self) -> &str;
    fn should_run_when_not_focused(&self) -> bool;
    fn get_jobs_id_to_wait(&self) -> Vec<JobId>;
    fn init(&mut self);
    fn run(&mut self, is_focused: bool) -> bool;
    fn uninit(&mut self);
}
impl_downcast!(Phase);

struct SystemData {
    system: SystemBoxed,
    dependencies: Vec<SystemId>,
}

pub struct PhaseWithSystems {
    name: String,
    systems_data: HashMap<SystemId, SystemData>,
    systems_running: Vec<SystemId>,
    systems_to_add: Vec<SystemId>,
    systems_to_remove: Vec<SystemId>,
}

impl PhaseWithSystems {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            systems_data: HashMap::new(),
            systems_running: Vec::new(),
            systems_to_add: Vec::new(),
            systems_to_remove: Vec::new(),
        }
    }

    pub fn get_system_mut<S>(&mut self) -> Option<&mut S>
    where
        S: System + Sized + 'static,
    {
        if let Some(system_data) = self.systems_data.get_mut(&S::id()) {
            return system_data.system.as_mut().downcast_mut();
        }
        None
    }

    pub fn get_system<S>(&self, system_id: &SystemId) -> Option<&S>
    where
        S: System + Sized + 'static,
    {
        if let Some(system_data) = self.systems_data.get(system_id) {
            return system_data.system.as_ref().downcast_ref();
        }
        None
    }

    pub fn add_system<S: System + 'static>(&mut self, system: S) -> &mut Self {
        let id = S::id();
        if let Entry::Vacant(e) = self.systems_data.entry(id) {
            self.systems_to_add.push(id);
            e.insert(SystemData {
                system: Box::new(system),
                dependencies: Vec::new(),
            });
        } else {
            eprintln!(
                "Trying to add twice a System with id {:?} in this Phase",
                id,
            );
        }
        self
    }

    pub fn remove_system(&mut self, system_id: &SystemId) -> &mut Self {
        if !self.systems_data.contains_key(system_id) {
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

    fn execute_systems(&mut self, is_focused: bool) -> bool {
        inox_profiler::scoped_profile!("phase::execute_systems");
        let mut can_continue = true;
        self.systems_running.iter().for_each(|id| {
            if let Some(system_data) = self.systems_data.get_mut(id) {
                inox_profiler::scoped_profile!(format!(
                    "{} {:?}",
                    "phase::execute_system",
                    system_data.system.get_name()
                )
                .as_str());
                if is_focused || system_data.system.should_run_when_not_focused() {
                    can_continue &= system_data.system.run();
                }
            }
        });
        can_continue
    }

    fn remove_pending_systems_from_execution(&mut self) -> &mut Self {
        for id in self.systems_to_remove.iter() {
            if let Some(index) = self.systems_running.iter().position(|s| s == id) {
                self.systems_running.remove(index);
            }
            if let Some(mut system_data) = self.systems_data.remove(id) {
                system_data.system.as_mut().uninit();
            }
        }
        self.systems_to_remove.clear();
        self
    }

    fn add_pending_systems_into_execution(&mut self) -> &mut Self {
        for id in self.systems_to_add.iter() {
            if let Some(system_data) = self.systems_data.get_mut(id) {
                system_data.system.init();
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
            if let Some(system_data) = self.systems_data.get(id) {
                if system_data.system.should_run_when_not_focused() {
                    return true;
                }
            }
        }
        false
    }

    fn init(&mut self) {
        self.add_pending_systems_into_execution();
    }

    fn run(&mut self, is_focused: bool) -> bool {
        self.remove_pending_systems_from_execution()
            .add_pending_systems_into_execution()
            .execute_systems(is_focused)
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
