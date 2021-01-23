use std::collections::HashMap;

use super::stage::*;

pub struct Scheduler {
    phases: Vec<String>,
    stages: HashMap<String, Box<dyn Stage>>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        let mut scheduler = Self {
            phases: Vec::new(),
            stages: HashMap::default(),
        };
        scheduler.create_default_execution_order();
        scheduler
    }

    pub fn create_default_execution_order(&mut self) -> &mut Self {
        self.phases.push(String::from(super::phases::INIT));
        self.phases.push(String::from(super::phases::BEGIN_FRAME));
        self.phases.push(String::from(super::phases::PRE_UPDATE));
        self.phases.push(String::from(super::phases::UPDATE));
        self.phases.push(String::from(super::phases::POST_UPDATE));
        self.phases.push(String::from(super::phases::PRE_RENDER));
        self.phases.push(String::from(super::phases::RENDER));
        self.phases.push(String::from(super::phases::POST_RENDER));
        self.phases.push(String::from(super::phases::END_FRAME));
        self.phases.push(String::from(super::phases::UNINIT));
        self
    }

    pub fn add_phase_after(&mut self, previous_phase_name: &str, phase_name: &str) -> &mut Self {
        let phase_index:i32 = self
            .phases
            .iter()
            .enumerate()
            .find(|(_i, name)| *name == previous_phase_name)
            .map(|(i, _)| i as _)
            .unwrap_or(-1);
        if phase_index >= 0 && phase_index < self.phases.len() as _ {
            self.phases.insert((phase_index + 1) as _, phase_name.to_string());
        }
        else {
            eprintln!("Previous Stage witn name {} does not exist", previous_phase_name);
        }
        self
    }

    pub fn add_phase_before(&mut self, previous_phase_name: &str, phase_name: &str) -> &mut Self {
        let phase_index:i32 = self
            .phases
            .iter()
            .enumerate()
            .find(|(_i, name)| *name == previous_phase_name)
            .map(|(i, _)| i as _)
            .unwrap_or(-1);
        if phase_index >= 0 && phase_index < self.phases.len() as _ {
            self.phases.insert(phase_index as _, phase_name.to_string());
        }
        else {
            eprintln!("Previous Stage witn name {} does not exist", previous_phase_name);
        }
        self
    }

    pub fn add_stage<S: Stage>(&mut self, stage_name: &str, stage:S) -> &mut Self {
        self.stages.insert(String::from(stage_name), Box::new(stage));
        self
    }

    pub fn get_stage<S:Stage>(&self, stage_name:&str) -> &S {
        self.stages
            .get(stage_name)
            .and_then(|stage| stage.downcast_ref::<S>())
        .unwrap_or_else(|| 
            panic!("Trying to retrieve a Stage {} that does not exist", stage_name)
        )
    }

    pub fn get_stage_mut<S:Stage>(&mut self, stage_name:&str) -> &mut S {
        self.stages
            .get_mut(stage_name)
            .and_then(|stage| stage.downcast_mut::<S>())
        .unwrap_or_else(|| 
            panic!("Trying to retrieve a Stage {} that does not exist", stage_name)
        )
    }

    pub fn run_once(&mut self) {
        for name in self.phases.iter() {
            if let Some(stage) = self.stages.get_mut(name) {
                stage.run();
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