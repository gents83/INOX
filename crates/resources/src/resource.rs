use sabi_serialize::Uid;
use std::{
    any::Any,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{SharedData, SharedDataRc};

pub type ResourceId = Uid;

pub trait ResourceTrait: Send + Sync + 'static
where
    Self::OnCreateData: Clone,
{
    type OnCreateData;

    fn on_create_resource(
        &mut self,
        shared_data: &SharedDataRc,
        id: &ResourceId,
        on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) where
        Self: Sized;
    fn on_destroy_resource(&mut self, shared_data: &SharedData, id: &ResourceId);
    fn on_copy_resource(&mut self, other: &Self);
}

pub trait GenericResourceTrait: Send + Sync + Any {
    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

#[derive(Clone)]
pub struct ResourceHandle<T>
where
    T: ResourceTrait,
{
    id: ResourceId,
    data: Arc<RwLock<T>>,
}

impl<T> ResourceHandle<T>
where
    T: ResourceTrait,
{
    #[inline]
    pub fn new(id: ResourceId, data: T) -> Self {
        Self {
            id,
            data: Arc::new(RwLock::new(data)),
        }
    }
    #[inline]
    pub fn id(&self) -> &ResourceId {
        &self.id
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

impl<T> GenericResourceTrait for ResourceHandle<T>
where
    T: ResourceTrait,
{
    #[inline]
    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

pub type Resource<T> = Arc<ResourceHandle<T>>;
pub type GenericResource = Arc<dyn GenericResourceTrait>;
pub type Handle<T> = Option<Resource<T>>;

pub trait ResourceCastTo {
    fn of_type<T: ResourceTrait>(&self) -> Resource<T>;
}

impl ResourceCastTo for GenericResource {
    #[inline]
    fn of_type<T: ResourceTrait>(&self) -> Resource<T> {
        let any = Arc::into_raw(self.clone().as_any());
        Arc::downcast(unsafe { Arc::from_raw(any) }).unwrap()
    }
}

pub fn swap_resource<T>(resource: &Resource<T>, other: &Resource<T>)
where
    T: ResourceTrait,
{
    resource
        .data
        .write()
        .unwrap()
        .on_copy_resource(&other.data.read().unwrap());
}
