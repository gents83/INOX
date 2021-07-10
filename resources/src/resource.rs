use nrg_messenger::implement_message;
use nrg_serialize::Uid;
use std::{
    any::TypeId,
    path::PathBuf,
    sync::{Arc, RwLock},
};

pub type ResourceId = Uid;
pub type Resource = Arc<RwLock<dyn ResourceTrait>>;
pub type ResourceOf<T> = Arc<RwLock<T>>;

pub trait ResourceTrait: Send + Sync + 'static {
    fn id(&self) -> ResourceId;
    fn path(&self) -> PathBuf;
}

pub trait ResourceBase {
    fn default<T>() -> Resource
    where
        T: ResourceTrait + Default,
    {
        Arc::new(RwLock::new(T::default()))
    }
    fn get<'a, T>(&self) -> &'a T
    where
        T: ResourceTrait + Sized;
    fn get_mut<'a, T>(&self) -> &'a mut T
    where
        T: ResourceTrait + Sized;
}

impl ResourceBase for Resource {
    fn get<'a, T>(&self) -> &'a T
    where
        T: ResourceTrait + Sized,
    {
        let resource = &*self.read().unwrap() as *const dyn ResourceTrait as *const T;
        unsafe { &*resource }
    }
    fn get_mut<'a, T>(&self) -> &'a mut T
    where
        T: ResourceTrait + Sized,
    {
        let resource = &mut *self.write().unwrap() as *mut dyn ResourceTrait as *mut T;
        unsafe { &mut *resource }
    }
}

#[derive(Clone)]
pub enum ResourceEvent {
    Reload(PathBuf),
    Remove(TypeId, Uid),
}
implement_message!(ResourceEvent);
