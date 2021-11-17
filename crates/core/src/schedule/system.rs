use std::any::{type_name, Any};

use sabi_serialize::{generate_uid_from_string, Uid};

pub type SystemId = Uid;

pub trait System: Send + Sync + Any {
    fn id() -> SystemId
    where
        Self: Sized,
    {
        generate_uid_from_string(Self::name())
    }
    fn name() -> &'static str
    where
        Self: Sized,
    {
        type_name::<Self>()
    }
    fn get_name(&self) -> &'static str {
        type_name::<Self>()
    }
    fn read_config(&mut self, plugin_name: &str);
    fn should_run_when_not_focused(&self) -> bool;
    fn init(&mut self);
    fn run(&mut self) -> bool;
    fn uninit(&mut self);
}

pub type SystemBoxed = Box<dyn System>;
