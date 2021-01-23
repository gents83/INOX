use std::collections::HashSet;
use downcast_rs::{impl_downcast, Downcast};
use super::system::*;

pub trait Stage: Downcast + Send + Sync {
    fn init(&mut self);
    fn run(&mut self);
    fn uninit(&mut self);
}
impl_downcast!(Stage);

pub struct SystemStage {
    systems: HashSet<SystemId>,
    systems_running: Vec<SystemBoxed>,
    systems_to_add: Vec<SystemBoxed>,
    systems_to_remove: Vec<SystemBoxed>,
}

impl Default for SystemStage {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemStage {
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

    pub fn remove_system<S: System<In = (), Out = ()>>(&mut self, system: S) -> &mut Self {
        self.remove_system_boxed(Box::new(system));
        self
    }

    fn execute_systems(&mut self) -> &mut Self {
        for s in self.systems_to_add.iter_mut() {
            s.run(());
        }
        self
    }

    fn remove_systems_from_execution(&mut self) -> &mut Self {
        for (i, s) in self.systems_to_remove.iter_mut().enumerate() {
            s.uninit();
            self.systems.remove(&s.id());
            self.systems_running.remove(i);
        }
        self.systems_to_remove.clear();
        self
    }

    fn add_systems_into_execution(&mut self) -> &mut Self {
        for s in self.systems_to_add.iter_mut() {
            self.systems.insert(s.id());
            s.init();
        }
        self.systems_running.append(&mut self.systems_to_add);
        self
    }

    fn add_system_boxed(&mut self, system:SystemBoxed) -> &mut Self {
        if self.systems.contains(&system.id()) {
            eprintln!("Trying to add twice a System with id {:?} from this Stage", system.id());
        } 
        else {
            self.systems_to_add.push(system);
        }
        self
    }

    fn remove_system_boxed(&mut self, system:SystemBoxed) -> &mut Self {
        if !self.systems.contains(&system.id()) {
            eprintln!("Trying to remove a System with id {:?} that is not in this Stage", system.id());
        } 
        else {
            self.systems_to_remove.push(system);
        }
        self
    }
}


impl Stage for SystemStage {
    fn init(&mut self) {
        println!("Executing init() for SystemStage");
    }

    fn run(&mut self) {
        println!("Executing systems for SystemStage");
        
        self.execute_systems()
            .remove_systems_from_execution()
            .add_systems_into_execution();
    }
    
    fn uninit(&mut self) {
        println!("Executing uninit() for SystemStage");
    }
    

}