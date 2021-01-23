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

impl Scheduler {
    pub fn new() -> Self {
        let mut scheduler = Self {
            phases_order: Vec::new(),
            phases: HashMap::default(),
        };
        scheduler.create_default_execution_order();
        scheduler
    }

    pub fn create_default_execution_order(&mut self) -> &mut Self {
        self.phases_order.push(String::from(super::phases::INIT));
        self.phases_order.push(String::from(super::phases::BEGIN_FRAME));
        self.phases_order.push(String::from(super::phases::PRE_UPDATE));
        self.phases_order.push(String::from(super::phases::UPDATE));
        self.phases_order.push(String::from(super::phases::POST_UPDATE));
        self.phases_order.push(String::from(super::phases::PRE_RENDER));
        self.phases_order.push(String::from(super::phases::RENDER));
        self.phases_order.push(String::from(super::phases::POST_RENDER));
        self.phases_order.push(String::from(super::phases::END_FRAME));
        self.phases_order.push(String::from(super::phases::UNINIT));
        self
    }

    pub fn add_phase_after(&mut self, previous_phase_name: &str, phase_name: &str) -> &mut Self {
        let phase_index:i32 = self
            .phases_order
            .iter()
            .enumerate()
            .find(|(_i, name)| *name == previous_phase_name)
            .map(|(i, _)| i as _)
            .unwrap_or(-1);
        if phase_index >= 0 && phase_index < self.phases_order.len() as _ {
            self.phases_order.insert((phase_index + 1) as _, phase_name.to_string());
        }
        else {
            eprintln!("Previous Phase witn name {} does not exist", previous_phase_name);
        }
        self
    }

    pub fn add_phase_before(&mut self, previous_phase_name: &str, phase_name: &str) -> &mut Self {
        let phase_index:i32 = self
            .phases_order
            .iter()
            .enumerate()
            .find(|(_i, name)| *name == previous_phase_name)
            .map(|(i, _)| i as _)
            .unwrap_or(-1);
        if phase_index >= 0 && phase_index < self.phases_order.len() as _ {
            self.phases_order.insert(phase_index as _, phase_name.to_string());
        }
        else {
            eprintln!("Previous Phase witn name {} does not exist", previous_phase_name);
        }
        self
    }

    pub fn create_phase(&mut self, phase_name: &str) -> &mut Self {
        let phase = Box::new(PhaseWithSystems::new(phase_name));
        self.phases.insert(String::from(phase.get_name()), phase);
        self
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