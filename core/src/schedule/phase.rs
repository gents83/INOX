use downcast_rs::{impl_downcast, Downcast};
use std::collections::HashSet;

use crate::{System, SystemBoxed, SystemId};

pub trait Phase: Downcast + Send + Sync {
    fn get_name(&self) -> &str;
    fn should_run_when_not_focused(&self) -> bool;
    fn init(&mut self);
    fn run(&mut self, is_focused: bool) -> bool;
    fn uninit(&mut self);
}
impl_downcast!(Phase);

pub struct PhaseWithSystems {
    name: String,
    systems: HashSet<SystemId>,
    systems_running: Vec<SystemBoxed>,
    systems_to_add: Vec<SystemBoxed>,
    systems_to_remove: Vec<SystemId>,
}

impl PhaseWithSystems {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            systems: HashSet::new(),
            systems_running: Vec::new(),
            systems_to_add: Vec::new(),
            systems_to_remove: Vec::new(),
        }
    }

    pub fn add_system<S: System + 'static>(&mut self, system: S) -> &mut Self {
        if self.systems.contains(&system.id()) {
            eprintln!(
                "Trying to add twice a System with id {:?} in this Phase",
                system.id()
            );
        } else {
            self.systems_to_add.push(Box::new(system));
        }
        self
    }

    pub fn remove_system(&mut self, system_id: &SystemId) -> &mut Self {
        if !self.systems.contains(system_id) {
            eprintln!(
                "Trying to remove a System with id {:?} in this Phase",
                system_id
            );
        } else if !self.systems_to_remove.contains(system_id) {
            self.systems_to_remove.push(*system_id);
        }
        self
    }

    fn remove_all_systems(&mut self) -> &mut Self {
        for s in self.systems_running.iter() {
            if !self.systems_to_remove.contains(&s.id()) {
                self.systems_to_remove.push(s.id());
            }
        }
        self
    }

    fn execute_systems(&mut self, is_focused: bool) -> bool {
        nrg_profiler::scoped_profile!("phase::execute_systems");
        let mut can_continue = true;
        for s in self.systems_running.iter_mut() {
            nrg_profiler::scoped_profile!(format!(
                "{}[{:?}]",
                "phase::execute_system",
                s.as_mut().id()
            )
            .as_str());
            let ok = if is_focused || s.should_run_when_not_focused() {
                s.run()
            } else {
                true
            };
            can_continue &= ok;
        }
        can_continue
    }

    fn remove_pending_systems_from_execution(&mut self) -> &mut Self {
        for id in self.systems_to_remove.iter_mut() {
            if let Some(index) = self.systems_running.iter().position(|s| s.id() == *id) {
                let mut system = self.systems_running.remove(index);
                system.uninit();
            }
            self.systems.remove(id);
        }
        self.systems_to_remove.clear();
        self
    }

    fn add_pending_systems_into_execution(&mut self) -> &mut Self {
        for s in self.systems_to_add.iter_mut() {
            s.init();
            self.systems.insert(s.id());
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
        for s in self.systems_running.iter() {
            if s.should_run_when_not_focused() {
                return true;
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
}

unsafe impl Send for PhaseWithSystems {}
unsafe impl Sync for PhaseWithSystems {}
