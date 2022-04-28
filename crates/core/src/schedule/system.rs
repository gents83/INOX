use std::{
    any::{type_name, Any},
    sync::{Arc, RwLock},
};

use downcast_rs::{impl_downcast, Downcast};
use inox_uid::{generate_uid_from_string, Uid};

pub type SystemId = Uid;

pub trait System: Downcast + Send + Sync + Any {
    fn system_id() -> SystemId
    where
        Self: Sized,
    {
        generate_uid_from_string(type_name::<Self>())
    }
    fn id(&self) -> SystemId {
        generate_uid_from_string(type_name::<Self>())
    }
    fn name(&self) -> &'static str {
        type_name::<Self>()
    }
    fn read_config(&mut self, plugin_name: &str);
    fn should_run_when_not_focused(&self) -> bool;
    fn init(&mut self);
    fn run(&mut self) -> bool;
    fn uninit(&mut self);
}
impl_downcast!(System);

pub type SystemRw = Arc<RwLock<Box<dyn System>>>;
