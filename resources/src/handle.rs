use std::{marker::PhantomData, sync::Arc};

use nrg_serialize::INVALID_UID;

use crate::{ResourceCastTo, ResourceData, ResourceId, SharedDataRw, TypedStorage};

pub trait Handle: Send + Sync {}
impl<T> Handle for ResourceHandle<T> where T: ResourceData {}

pub struct ResourceHandle<T>
where
    T: ResourceData + ?Sized,
{
    id: ResourceId,
    shared_data: SharedDataRw,
    _marker: PhantomData<T>,
}

impl<T> Default for ResourceHandle<T>
where
    T: ResourceData,
{
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            shared_data: SharedDataRw::default(),
            _marker: PhantomData::default(),
        }
    }
}

impl<T> ResourceHandle<T>
where
    T: ResourceData,
{
    pub fn new(id: ResourceId, shared_data: SharedDataRw) -> Self {
        Self {
            id,
            shared_data,
            _marker: PhantomData::default(),
        }
    }

    pub fn id(&self) -> ResourceId {
        self.id
    }

    pub fn get(&self) -> &T {
        let mut shared_data = self.shared_data.write().unwrap();
        let resource = shared_data.get_storage::<T>().resource(self.id).to::<T>();
        let value = resource.as_ref() as *const T;
        unsafe { &*value }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn get_mut(&self) -> &mut T {
        let mut shared_data = self.shared_data.write().unwrap();
        let resource = shared_data.get_storage::<T>().resource(self.id).to::<T>();
        let value = resource.as_mut() as *mut T;
        unsafe { &mut *value }
    }
}

pub type ResourceRef<T> = Box<Arc<ResourceHandle<T>>>;
pub type GenericRef = Box<Arc<dyn Handle>>;

pub trait HandleCastTo {
    fn from_generic<T: ResourceData>(&mut self) -> ResourceRef<T>;
}
pub trait HandleCastFrom {
    fn as_generic(&mut self) -> GenericRef;
}

impl HandleCastTo for GenericRef {
    fn from_generic<T: ResourceData>(&mut self) -> ResourceRef<T> {
        let resource_data = self.as_mut() as *mut Arc<dyn Handle> as *mut Arc<ResourceHandle<T>>;
        unsafe { Box::from_raw(resource_data) }
    }
}

impl<T> HandleCastFrom for ResourceRef<T>
where
    T: ResourceData,
{
    fn as_generic(&mut self) -> GenericRef {
        let resource_data = self.as_mut() as *mut Arc<ResourceHandle<T>> as *mut Arc<dyn Handle>;
        unsafe { Box::from_raw(resource_data) }
    }
}
