use inox_uid::Uid;
use std::{
    any::Any,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

pub const DEBUG_RESOURCES: bool = false;

pub type ResourceId = Uid;

pub trait Function<T>: FnMut(&mut T)
where
    T: ResourceTrait,
{
    fn as_boxed(&self) -> Box<dyn Function<T>>;
}
impl<F, T> Function<T> for F
where
    F: 'static + FnMut(&mut T) + Clone,
    T: ResourceTrait,
{
    fn as_boxed(&self) -> Box<dyn Function<T>> {
        Box::new(self.clone())
    }
}
impl<T> Clone for Box<dyn Function<T>>
where
    T: ResourceTrait,
{
    fn clone(&self) -> Self {
        (**self).as_boxed()
    }
}

#[derive(Clone)]
pub struct OnCreateData<T>
where
    T: ResourceTrait,
{
    func: Vec<Box<dyn Function<T>>>,
}
unsafe impl<T> Send for OnCreateData<T> where T: ResourceTrait {}
unsafe impl<T> Sync for OnCreateData<T> where T: ResourceTrait {}

impl<T> OnCreateData<T>
where
    T: ResourceTrait,
{
    pub fn create<F>(f: F) -> Option<Self>
    where
        F: Function<T> + 'static,
    {
        Some(Self {
            func: vec![Box::new(f)],
        })
    }

    pub fn call_func(&mut self, r: &mut T) {
        self.func.iter_mut().for_each(|f| {
            f(r);
        });
    }
}

pub trait ResourceTrait: Send + Sync {
    fn is_initialized(&self) -> bool;
    fn invalidate(&mut self) -> &mut Self;
    fn typename() -> &'static str {
        std::any::type_name::<Self>()
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
    T: ResourceTrait + Clone,
{
    inox_profiler::scoped_profile!("swap_resource");
    let new = {
        let o = other.data.read().unwrap();
        o.clone()
    };
    {
        let old = &mut *resource.data.write().unwrap();
        *old = new;
        old.invalidate();
    }
    if DEBUG_RESOURCES {
        inox_log::debug_log!(
            "Swapping resource {:?} with id {:?}",
            T::typename(),
            resource.id()
        );
    }
}
