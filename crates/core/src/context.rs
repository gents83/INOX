use std::sync::{Arc, RwLockReadGuard, RwLockWriteGuard};

use inox_messenger::MessageHubRc;
use inox_resources::SharedDataRc;
use inox_time::{Timer, TimerRw};

use crate::{JobHandlerRw, Phases, Scheduler, SchedulerRw, System, SystemEvent, SystemId};

#[derive(Default)]
pub struct Context {
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    global_timer: TimerRw,
    job_handler: JobHandlerRw,
    scheduler: SchedulerRw,
}

impl Context {
    pub fn shared_data(&self) -> &SharedDataRc {
        &self.shared_data
    }
    pub fn message_hub(&self) -> &MessageHubRc {
        &self.message_hub
    }
    pub fn global_timer(&self) -> RwLockReadGuard<Timer> {
        self.global_timer.read().unwrap()
    }
    pub fn global_timer_mut(&self) -> RwLockWriteGuard<Timer> {
        self.global_timer.write().unwrap()
    }
    pub(crate) fn scheduler_mut(&self) -> RwLockWriteGuard<Scheduler> {
        self.scheduler.write().unwrap()
    }
    pub fn job_handler(&self) -> &JobHandlerRw {
        &self.job_handler
    }
    pub fn add_system<S>(&self, phase: Phases, system: S, dependencies: Option<&[SystemId]>)
    where
        S: System + 'static,
    {
        let system = Box::new(system);
        let id = system.id();
        self.scheduler_mut()
            .add_system(phase, system, dependencies, &self.job_handler);
        self.message_hub.send_event(SystemEvent::Added(id, phase));
    }
    pub fn remove_system(&self, phase: Phases, system_id: &SystemId) {
        self.scheduler_mut().remove_system(phase, system_id);
        self.message_hub
            .send_event(SystemEvent::Removed(*system_id, phase));
    }
}

pub type ContextRc = Arc<Context>;
