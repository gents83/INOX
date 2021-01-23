use std::collections::HashSet;
use super::system::*;

pub struct Stage {
    systems: HashSet<SystemId>,
    systems_running: Vec<SystemBoxed>,
    systems_to_add: Vec<SystemBoxed>,
    systems_to_remove: Vec<SystemBoxed>,
}

impl Default for Stage {
    fn default() -> Self {
        Self::new()
    }
}

impl Stage {
    pub fn new() -> Self {
        Self {
            systems: HashSet::new(),
            systems_running: Vec::new(),
            systems_to_add: Vec::new(),
            systems_to_remove: Vec::new(),
        }
    }

    pub fn add_system<S: System<In = (), Out = ()>>(&mut self, system: S) -> &mut Self {
        self.add_system_boxed(Box::new(system));
        self
    }

    fn add_system_boxed(&mut self, system:SystemBoxed) -> &mut Self {
        if self.systems.contains(&system.id()) {
            eprintln!("Trying to add twice a System with id {:?}", system.id());
        } 
        else {
            self.systems.insert(system.id());
            self.systems_to_add.push(system);
        }
        self
    }
}