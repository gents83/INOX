use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub trait Event: Send + Sync {}

type FrameEvent = (u64, Arc<dyn Event>);

pub struct Events {
    frame: u64,
    map: HashMap<TypeId, Vec<FrameEvent>>,
}

impl Default for Events {
    fn default() -> Self {
        Self {
            frame: 0,
            map: HashMap::new(),
        }
    }
}

impl Events {
    pub fn register_event<T>(&mut self)
    where
        T: Event + 'static,
    {
        self.map.insert(TypeId::of::<T>(), Vec::new());
    }

    pub fn unregister_event<T>(&mut self)
    where
        T: Event + 'static,
    {
        self.map.remove(&TypeId::of::<T>());
    }

    pub fn send_event<T>(&mut self, event: T)
    where
        T: Event + 'static,
    {
        self.map
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .push((self.frame, Arc::new(event)));
    }

    pub fn read_events<T>(&self) -> Vec<&T>
    where
        T: Event + 'static,
    {
        let map =
            |i: &(u64, Arc<dyn Event>)| unsafe { &*Arc::into_raw(std::mem::transmute_copy(&i.1)) };
        self.map
            .get(&TypeId::of::<T>())
            .unwrap()
            .iter()
            .map(map)
            .collect()
    }

    pub fn update(&mut self, frame_count: u64) {
        for (_id, map) in self.map.iter_mut() {
            map.retain(|(frame, _el)| *frame == frame_count);
        }
    }
}

pub type EventsRw = Arc<RwLock<Events>>;
