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
    pub fn send_event<T>(&mut self, event: T)
    where
        T: Event + 'static + Sized,
    {
        if let Some(list) = self.map.get_mut(&TypeId::of::<T>()) {
            list.push((self.frame, Arc::new(event)));
        } else {
            self.map.insert(TypeId::of::<T>(), Vec::new());
            self.send_event(event);
        }
    }
    pub fn push_event_to_next_frame<T>(&mut self, event: T)
    where
        T: Event + 'static + Sized,
    {
        if let Some(list) = self.map.get_mut(&TypeId::of::<T>()) {
            list.push((self.frame + 1, Arc::new(event)));
        } else {
            self.map.insert(TypeId::of::<T>(), Vec::new());
            self.push_event_to_next_frame(event);
        }
    }

    pub fn read_events<T>(&self) -> Option<Vec<&T>>
    where
        T: Event + 'static,
    {
        if let Some(list) = self.map.get(&TypeId::of::<T>()) {
            let map = |i: &(u64, Arc<dyn Event>)| unsafe {
                &*Arc::into_raw(std::mem::transmute_copy(&i.1))
            };
            Some(list.iter().map(map).collect())
        } else {
            None
        }
    }

    pub fn update(&mut self, frame_count: u64) {
        for (_id, map) in self.map.iter_mut() {
            map.retain(|(frame, _el)| *frame >= frame_count);
        }
        self.frame = frame_count;
    }
}

pub type EventsRw = Arc<RwLock<Events>>;
