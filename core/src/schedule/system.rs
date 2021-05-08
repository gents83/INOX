use std::time::{SystemTime, UNIX_EPOCH};

use crate::Job;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SystemId(pub u64);

impl Default for SystemId {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemId {
    pub fn new() -> Self {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as _;
        SystemId(secs)
    }
}

pub trait System: Send + Sync {
    fn id(&self) -> SystemId;

    fn init(&mut self);
    fn run(&mut self) -> (bool, Vec<Job>);
    fn uninit(&mut self);
}

pub type SystemBoxed = Box<dyn System>;
