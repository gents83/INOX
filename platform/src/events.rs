use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub trait Event: Send + Sync {
    fn get_frame(&self) -> u64;
}

pub struct Events {
    list: HashMap<TypeId, Vec<Arc<dyn Event>>>,
}

impl Default for Events {
    fn default() -> Self {
        Self {
            list: HashMap::new(),
        }
    }
}

impl Events {
    pub fn register_event<T>(&mut self)
    where
        T: Event + 'static,
    {
        self.list.insert(TypeId::of::<T>(), Vec::new());
    }

    pub fn unregister_event<T>(&mut self)
    where
        T: Event + 'static,
    {
        self.list.remove(&TypeId::of::<T>());
    }

    pub fn send_event<T>(&mut self, event: T)
    where
        T: Event + 'static,
    {
        self.list
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .push(Arc::new(event));
    }

    pub fn read_events<T>(&self) -> Vec<&T>
    where
        T: Event + 'static,
    {
        let map = |i: &Arc<dyn Event>| unsafe { &*Arc::into_raw(std::mem::transmute_copy(i)) };
        self.list
            .get(&TypeId::of::<T>())
            .unwrap()
            .iter()
            .map(map)
            .collect()
    }

    pub fn clear_events<T>(&mut self, frame_count: u64)
    where
        T: Event + 'static,
    {
        self.list
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .retain(|el| el.get_frame() == frame_count);
    }
}

pub type EventsRw = Arc<RwLock<Events>>;
