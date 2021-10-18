use std::any::type_name;

use nrg_serialize::{generate_uid_from_string, Uid};

pub type SystemId = Uid;

pub trait System: Send + Sync {
    fn id(&self) -> SystemId {
        generate_uid_from_string(self.name())
    }
    fn name(&self) -> &'static str {
        type_name::<Self>()
    }
    fn should_run_when_not_focused(&self) -> bool;
    fn init(&mut self);
    fn run(&mut self) -> bool;
    fn uninit(&mut self);
}

pub type SystemBoxed = Box<dyn System>;
