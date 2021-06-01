use nrg_messenger::implement_message;
use nrg_serialize::Uid;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

pub type ResourceId = Uid;

pub trait ResourceTrait {
    fn id(&self) -> ResourceId;
    fn path(&self) -> PathBuf;
}

#[derive(Clone)]
pub enum ResourceEvent {
    Reload(PathBuf),
}
implement_message!(ResourceEvent);

pub struct Resource<T>
where
    T: ResourceTrait,
{
    data: RwLock<T>,
}
unsafe impl<T> Send for Resource<T> where T: ResourceTrait {}
unsafe impl<T> Sync for Resource<T> where T: ResourceTrait {}

impl<T> Resource<T>
where
    T: ResourceTrait,
{
    #[inline]
    pub fn new(data: T) -> Self {
        Self {
            data: RwLock::new(data),
        }
    }
    #[inline]
    pub fn get(&self) -> RwLockReadGuard<'_, T> {
        self.data.read().unwrap()
    }
    #[inline]
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, T> {
        self.data.write().unwrap()
    }
}

impl<T> ResourceTrait for Resource<T>
where
    T: ResourceTrait + Sized + 'static,
{
    #[inline]
    fn id(&self) -> ResourceId {
        self.get().id()
    }
    #[inline]
    fn path(&self) -> PathBuf {
        self.get().path()
    }
}

pub type ResourceTraitRc = Arc<Box<dyn ResourceTrait>>;
pub type ResourceRc<T> = Arc<Box<Resource<T>>>;
