use std::{
    any::{type_name, Any},
    sync::{Arc, RwLock},
};

use downcast_rs::{impl_downcast, Downcast};
use inox_uid::Uid;

pub type SystemId = Uid;

pub trait SystemUID {
    fn system_id() -> SystemId
    where
        Self: Sized;
}
#[macro_export]
macro_rules! implement_unique_system_uid {
    ($Type:ident) => {
        impl $crate::SystemUID for $Type {
            fn system_id() -> $crate::SystemId
            where
                Self: Sized,
            {
                inox_uid::generate_uid_from_string(std::any::type_name::<Self>())
            }
        }
    };
    ($Type:ident<$Lifetime:lifetime>) => {
        impl $crate::SystemUID for $Type<$Lifetime> {
            fn system_id() -> $crate::SystemId
            where
                Self: Sized,
            {
                inox_uid::generate_uid_from_string(std::any::type_name::<Self>())
            }
        }
    };
}

pub trait System: Downcast + Send + Sync + Any + SystemUID {
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
