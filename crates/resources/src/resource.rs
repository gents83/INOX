use inox_messenger::MessageHubRc;
use inox_uid::Uid;
use std::{
    any::Any,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{ResourceEvent, SharedData, SharedDataRc};

pub type ResourceId = Uid;

pub trait ResourceTrait: Send + Sync + 'static
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

    fn on_create_resource(
        &mut self,
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: &ResourceId,
        on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) where
        Self: Sized,
    {
        self.on_create(shared_data, message_hub, id, on_create_data);
        message_hub.send_event(ResourceEvent::<Self>::Created(
            shared_data.get_resource(id).unwrap(),
        ));
    }
    fn on_destroy_resource(
        &mut self,
        shared_data: &SharedData,
        message_hub: &MessageHubRc,
        id: &ResourceId,
    ) where
        Self: Sized,
    {
        message_hub.send_event(ResourceEvent::<Self>::Destroyed(*id));
        self.on_destroy(shared_data, message_hub, id);
    }
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
    inox_profiler::scoped_profile!("swap_resource");
    resource
        .data
        .write()
        .unwrap()
        .on_copy(&other.data.read().unwrap());
}
