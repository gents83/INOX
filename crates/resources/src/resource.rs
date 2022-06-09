use inox_messenger::MessageHubRc;
use inox_uid::Uid;
use std::{
    any::Any,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{SharedData, SharedDataRc};

pub type ResourceId = Uid;

pub trait ResourceTrait: Send + Sync
where
    Self::OnCreateData: Send + Sync + Clone,
{
    type OnCreateData;
    fn on_create(
        &mut self,
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: &ResourceId,
        on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    );
    fn on_copy(&mut self, other: &Self)
    where
        Self: Sized;
    fn on_destroy(&mut self, shared_data: &SharedData, message_hub: &MessageHubRc, id: &ResourceId);
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
        inox_profiler::scoped_profile!(
            "Resource<{}>::get",
            std::any::type_name::<T>()
                .split(':')
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
        );
        self.data.read().unwrap()
    }

    #[inline]
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, T> {
        inox_profiler::scoped_profile!(
            "Resource<{}>::get_mut",
            std::any::type_name::<T>()
                .split(':')
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
        );
        self.data.write().unwrap()
    }
}

impl<T> GenericResourceTrait for ResourceHandle<T>
where
    T: ResourceTrait + 'static,
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
    fn of_type<T>(&self) -> Resource<T>
    where
        T: ResourceTrait + 'static;
}

impl ResourceCastTo for GenericResource {
    #[inline]
    fn of_type<T>(&self) -> Resource<T>
    where
        T: ResourceTrait + 'static,
    {
        let any = Arc::into_raw(self.clone().as_any());
        Arc::downcast(unsafe { Arc::from_raw(any) }).unwrap()
    }
}

pub fn swap_resource<T>(resource: &Resource<T>, other: &Resource<T>)
where
    T: ResourceTrait,
{
    inox_profiler::scoped_profile!("swap_resource");
    resource
        .data
        .write()
        .unwrap()
        .on_copy(&other.data.read().unwrap());
}
