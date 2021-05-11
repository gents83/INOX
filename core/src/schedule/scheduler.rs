use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::Sender,
        Arc, Mutex,
    },
};

use crate::{Job, Phase, PhaseWithSystems};

pub struct Scheduler {
    is_running: bool,
    is_started: bool,
    phases_order: Vec<String>,
    phases: HashMap<String, Box<dyn Phase>>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Sync for Scheduler {}
unsafe impl Send for Scheduler {}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            is_running: true,
            is_started: false,
            phases_order: Vec::new(),
            phases: HashMap::default(),
        }
    }

    pub fn cancel(&mut self) {
        self.is_running = false;
    }

    pub fn uninit(&mut self) {
        self.cancel();
        for name in self.phases_order.iter() {
            if let Some(phase) = self.phases.get_mut(name) {
                phase.uninit();
            }
        }
    }

    fn append_phase(&mut self, phase_name: &str) -> &mut Self {
        self.phases_order.push(String::from(phase_name));
        self
    }

    fn remove_phase(&mut self, phase_name: &str) -> &mut Self {
        let index = self.get_phase_index(phase_name);
        if index >= 0 {
            self.phases_order.remove(index as _);
        }
        self
    }

    #[allow(dead_code)]
    pub fn add_phase_after(&mut self, previous_phase_name: &str, phase_name: &str) -> &mut Self {
        let phase_index: i32 = self.get_phase_index(previous_phase_name);
        if phase_index >= 0 && phase_index < self.phases_order.len() as _ {
            self.phases_order
                .insert((phase_index + 1) as _, phase_name.to_string());
        } else {
            eprintln!(
                "Previous Phase witn name {} does not exist",
                previous_phase_name
            );
        }
        self
    }

    pub fn add_phase_before(&mut self, previous_phase_name: &str, phase_name: &str) -> &mut Self {
        let phase_index: i32 = self.get_phase_index(previous_phase_name);
        if phase_index >= 0 && phase_index < self.phases_order.len() as _ {
            self.phases_order
                .insert(phase_index as _, phase_name.to_string());
        } else {
            eprintln!(
                "Next Phase witn name {} does not exist",
                previous_phase_name
            );
        }
        self
    }

    pub fn create_phase_before<S: Phase>(
        &mut self,
        mut phase: S,
        previous_phase_name: &str,
    ) -> &mut Self {
        if self.get_phase_index(phase.get_name()) < 0 {
            self.add_phase_before(previous_phase_name, phase.get_name());
        }
        phase.init();
        self.phases
            .insert(String::from(phase.get_name()), Box::new(phase));
        self.is_started = true;
        self
    }

    pub fn create_phase<S: Phase>(&mut self, mut phase: S) -> &mut Self {
        if self.get_phase_index(phase.get_name()) < 0 {
            self.append_phase(phase.get_name());
        }
        phase.init();
        self.phases
            .insert(String::from(phase.get_name()), Box::new(phase));
        self.is_started = true;
        self
    }

    pub fn create_phase_with_systems(&mut self, phase_name: &str) -> &mut Self {
        let phase = Box::new(PhaseWithSystems::new(phase_name));
        if self.get_phase_index(phase_name) < 0 {
            self.append_phase(phase_name);
        }
        self.phases.insert(String::from(phase.get_name()), phase);
        self.get_phase_mut::<PhaseWithSystems>(phase_name).init();
        self.is_started = true;
        self
    }

    pub fn destroy_phase(&mut self, phase_name: &str) -> &mut Self {
        self.remove_phase(phase_name);
        self.phases.retain(|name, boxed_phase| {
            if name == phase_name {
                boxed_phase.uninit();
                false
            } else {
                true
            }
        });
        if self.phases.is_empty() {
            self.is_started = false;
        }
        self
    }

    pub fn get_phase_index(&self, phase_name: &str) -> i32 {
        self.phases_order
            .iter()
            .enumerate()
            .find(|(_i, name)| *name == phase_name)
            .map(|(i, _)| i as _)
            .unwrap_or(-1)
    }

    pub fn get_phase<S: Phase>(&self, phase_name: &str) -> &S {
        self.phases
            .get(phase_name)
            .and_then(|phase| phase.downcast_ref::<S>())
            .unwrap_or_else(|| {
                panic!(
                    "Trying to retrieve a Phase {} that does not exist",
                    phase_name
                )
            })
    }

    pub fn get_phase_mut<S: Phase>(&mut self, phase_name: &str) -> &mut S {
        self.phases
            .get_mut(phase_name)
            .and_then(|phase| phase.downcast_mut::<S>())
            .unwrap_or_else(|| {
                panic!(
                    "Trying to retrieve a Phase {} that does not exist",
                    phase_name
                )
            })
    }

    pub fn run_once(&mut self, sender: Arc<Mutex<Sender<Job>>>) -> bool {
        if !self.is_started {
            return self.is_running;
        }
        nrg_profiler::scoped_profile!("scheduler::run_once");
        let mut can_continue = self.is_running;
        for name in self.phases_order.iter() {
            if let Some(phase) = self.phases.get_mut(name) {
                nrg_profiler::scoped_profile!(
                    format!("{}[{}]", "scheduler::run_phase", name).as_str()
                );
                let (ok, jobs) = phase.run();
                if !jobs.is_empty() {
                    let wait_count = Arc::new(AtomicUsize::new(jobs.len()));
                    for mut j in jobs {
                        j.set_wait_count(wait_count.clone());
                        let res = sender.lock().unwrap().send(j);
                        if res.is_err() {
                            panic!("Failed to add job to execution queue");
                        }
                    }

                    while wait_count.load(Ordering::SeqCst) > 0 {
                        thread::yield_now();
                    }
                }

                can_continue &= ok;
            }
        }
        can_continue
    }
}
