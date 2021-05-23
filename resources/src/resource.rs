use nrg_serialize::{generate_random_uid, Uid};
use std::{
    path::PathBuf,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

pub type ResourceId = Uid;

pub trait ResourceTrait {
    fn id(&self) -> ResourceId;
    fn path(&self) -> PathBuf;
}

pub struct Resource<T> {
    id: ResourceId,
    data: RwLock<T>,
}
unsafe impl<T> Send for Resource<T> {}
unsafe impl<T> Sync for Resource<T> {}

impl<T> Resource<T> {
    #[inline]
    pub fn new(data: T) -> Self {
        Self {
            id: generate_random_uid(),
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
    T: Sized + 'static,
{
    #[inline]
    fn id(&self) -> ResourceId {
        self.id
    }
    #[inline]
    fn path(&self) -> PathBuf {
        PathBuf::default()
    }
}

pub type ResourceTraitRc = Arc<Box<dyn ResourceTrait>>;
pub type ResourceRc<T> = Arc<Box<Resource<T>>>;
