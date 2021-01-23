use std::collections::HashMap;

use super::phase::*;

pub struct Scheduler {
    phases_order: Vec<String>,
    phases: HashMap<String, Box<dyn Phase>>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        for name in self.phases_order.iter() {
            if let Some(phase) = self.phases.get_mut(name) {
                phase.uninit();
            }
        }
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            phases_order: Vec::new(),
            phases: HashMap::default(),
        }
    }

    fn append_phase(&mut self, phase_name: &str) -> &mut Self {
        self.phases_order.push(String::from(phase_name));
        self
    }

    pub fn add_phase_after(&mut self, previous_phase_name: &str, phase_name: &str) -> &mut Self {
        let phase_index:i32 = self.get_phase_index(previous_phase_name);
        if phase_index >= 0 && phase_index < self.phases_order.len() as _ {
            self.phases_order.insert((phase_index + 1) as _, phase_name.to_string());
        }
        else {
            eprintln!("Previous Phase witn name {} does not exist", previous_phase_name);
        }
        self
    }

    pub fn add_phase_before(&mut self, previous_phase_name: &str, phase_name: &str) -> &mut Self {
        let phase_index:i32 = self.get_phase_index(previous_phase_name);
        if phase_index >= 0 && phase_index < self.phases_order.len() as _ {
            self.phases_order.insert(phase_index as _, phase_name.to_string());
        }
        else {
            eprintln!("Next Phase witn name {} does not exist", previous_phase_name);
        }
        self
    }

    pub fn create_phase<S: Phase>(&mut self, mut phase: S) -> &mut Self {
        if self.get_phase_index(phase.get_name()) < 0 {
            self.append_phase(phase.get_name());
        }
        phase.init();
        self.phases.insert(String::from(phase.get_name()), Box::new(phase));
        self
    }

    pub fn create_phase_with_systems(&mut self, phase_name: &str) -> &mut Self {
        let phase = Box::new(PhaseWithSystems::new(phase_name));
        if self.get_phase_index(phase_name) < 0 {
            self.append_phase(phase_name);
        }
        self.phases.insert(String::from(phase.get_name()), phase);
        self.get_phase_mut::<PhaseWithSystems>(phase_name).init();
        self
    }

    pub fn get_phase_index(&self, phase_name:&str) -> i32 {
        self.phases_order
            .iter()
            .enumerate()
            .find(|(_i, name)| *name == phase_name)
            .map(|(i, _)| i as _)
            .unwrap_or(-1)
    }

    pub fn get_phase<S:Phase>(&self, phase_name:&str) -> &S {
        self.phases
            .get(phase_name)
            .and_then(|phase| phase.downcast_ref::<S>())
        .unwrap_or_else(|| 
            panic!("Trying to retrieve a Phase {} that does not exist", phase_name)
        )
    }

    pub fn get_phase_mut<S:Phase>(&mut self, phase_name:&str) -> &mut S {
        self.phases
            .get_mut(phase_name)
            .and_then(|phase| phase.downcast_mut::<S>())
        .unwrap_or_else(|| 
            panic!("Trying to retrieve a Phase {} that does not exist", phase_name)
        )
    }

    pub fn run_once(&mut self) {
        for name in self.phases_order.iter() {
            if let Some(phase) = self.phases.get_mut(name) {
                phase.run();
            }
        }
    }
    
    pub fn run(&mut self) {        
        loop 
        {                
            let is_ended = false;
            self.run_once();
            if is_ended
            {
                break;
            }
        }
    }
}