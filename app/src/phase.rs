use std::collections::HashSet;
use downcast_rs::{impl_downcast, Downcast};
use super::system::*;

pub trait Phase: Downcast + Send + Sync {
    fn get_name(&self) -> &str;
    fn init(&mut self);
    fn run(&mut self);
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
            name : String::from(name),
            systems: HashSet::new(),
            systems_running: Vec::new(),
            systems_to_add: Vec::new(),
            systems_to_remove: Vec::new(),
        }
    }

    pub fn add_system<S: System<In = (), Out = ()>>(&mut self, system: S) -> &mut Self {        
        if self.systems.contains(&system.id()) {
            eprintln!("Trying to add twice a System with id {:?} in this Phase", system.id());
        } 
        else {
            self.systems_to_add.push(Box::new(system));
        }
        self
    }

    pub fn remove_system(&mut self, system_id: &SystemId) -> &mut Self {
        if !self.systems.contains(&system_id) {
            eprintln!("Trying to remove a System with id {:?} in this Phase", system_id);
        } 
        else {
            self.systems_to_remove.push(*system_id);
        }
        self
    }

    fn remove_all_systems(&mut self) -> &mut Self {
        for s in self.systems_running.iter() {
            self.systems_to_remove.push(s.id());
        }
        self
    }

    fn execute_systems(&mut self) -> &mut Self {
        for s in self.systems_running.iter_mut() {
            s.run(());
        }
        self
    }

    fn remove_pending_systems_from_execution(&mut self) -> &mut Self {
        for (i, s) in self.systems_to_remove.iter_mut().enumerate() {
            let mut system = self.systems_running.remove(i);
            system.uninit();
            self.systems.remove(&s);
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

    fn init(&mut self) {
        println!("Phase {} initialization...", self.get_name());

        self.add_pending_systems_into_execution();
    }

    fn run(&mut self) {
        println!("Phase {} systems executing...", self.get_name());
        
        self.remove_pending_systems_from_execution()
            .add_pending_systems_into_execution()
            .execute_systems();
    }
    
    fn uninit(&mut self) {
        println!("Phase {} uninitialization...", self.get_name());

        self.remove_all_systems()
        .remove_pending_systems_from_execution();
        
        self.systems_running.clear();
    }
}
