use std::{
    any::TypeId,
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

const VALUE_ZERO: usize = 0;
const VALUE_ONE: usize = 1;
const UNIQUE_BIT: usize = !(usize::max_value() >> VALUE_ONE);

pub struct Atomic(AtomicUsize);

impl Atomic {
    pub const fn new() -> Self {
        Self(AtomicUsize::new(0))
    }
    pub fn request_borrow(&self) -> bool {
        let result = self
            .0
            .fetch_add(VALUE_ONE, Ordering::Acquire)
            .wrapping_add(VALUE_ONE);
        debug_assert!(result != 0, "Invalid borrow request on atomic element");
        if result & UNIQUE_BIT != VALUE_ZERO {
            self.0.fetch_sub(VALUE_ONE, Ordering::Release);
            false
        } else {
            true
        }
    }
    pub fn request_borrow_mut(&self) -> bool {
        self.0
            .compare_exchange(VALUE_ZERO, UNIQUE_BIT, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }
    pub fn release_borrow(&self) {
        let result = self.0.fetch_sub(VALUE_ONE, Ordering::Release);
        debug_assert!(
            result != VALUE_ZERO,
            "Release borrow seems to be unbalanced"
        );
        debug_assert!(
            result & UNIQUE_BIT == VALUE_ZERO,
            "Releasing shared unique borrow"
        );
    }
    pub fn release_borrow_mut(&self) {
        let result = self.0.fetch_and(!UNIQUE_BIT, Ordering::Release);
        debug_assert_ne!(result & UNIQUE_BIT, 0, "Releasing shared unique borrow");
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ResourceId(pub TypeId);

pub trait ResourceTrait {
    fn id(&self) -> ResourceId;
    fn path(&self) -> PathBuf;
}

pub struct Resource<T> {
    data: UnsafeCell<T>,
    atomic: Atomic,
}
unsafe impl<T> Send for Resource<T> {}
unsafe impl<T> Sync for Resource<T> {}

impl<T> Resource<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            atomic: Atomic::new(),
        }
    }
    pub fn borrow(&self) -> ResourceRef<T> {
        ResourceRef::new(self)
    }
    pub fn borrow_mut(&self) -> ResourceRefMut<T> {
        ResourceRefMut::new(self)
    }
}

impl<T> ResourceTrait for Resource<T>
where
    T: Sized + 'static,
{
    fn id(&self) -> ResourceId {
        ResourceId(TypeId::of::<T>())
    }
    fn path(&self) -> PathBuf {
        PathBuf::default()
    }
}

pub struct ResourceRef<'a, T> {
    borrow: &'a Atomic,
    resource: &'a T,
}

impl<'a, T> ResourceRef<'a, T> {
    pub fn new(Resource { data, atomic }: &'a Resource<T>) -> Self {
        if atomic.request_borrow() {
            Self {
                borrow: atomic,
                resource: unsafe { &*data.get() },
            }
        } else {
            panic!(
                "Failed to acquire shared lock on resource: {}.",
                std::any::type_name::<T>()
            );
        }
    }
}

unsafe impl<T> Send for ResourceRef<'_, T> {}
unsafe impl<T> Sync for ResourceRef<'_, T> {}

impl<'a, T> Drop for ResourceRef<'a, T> {
    fn drop(&mut self) {
        self.borrow.release_borrow()
    }
}
impl<'a, T> Deref for ResourceRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.resource
    }
}

pub struct ResourceRefMut<'a, T> {
    borrow: &'a Atomic,
    resource: &'a mut T,
}

impl<'a, T> ResourceRefMut<'a, T> {
    pub fn new(Resource { data, atomic }: &'a Resource<T>) -> Self {
        if atomic.request_borrow_mut() {
            Self {
                borrow: atomic,
                resource: unsafe { &mut *data.get() },
            }
        } else {
            panic!(
                "Failed to acquire exclusive lock on resource: {}.",
                std::any::type_name::<T>()
            );
        }
    }
}

unsafe impl<T> Send for ResourceRefMut<'_, T> {}
unsafe impl<T> Sync for ResourceRefMut<'_, T> {}

impl<'a, T> Drop for ResourceRefMut<'a, T> {
    fn drop(&mut self) {
        self.borrow.release_borrow_mut()
    }
}
impl<'a, T> Deref for ResourceRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.resource
    }
}
impl<'a, T> DerefMut for ResourceRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.resource
    }
}

pub type ResourceBoxed = Box<dyn ResourceTrait>;
