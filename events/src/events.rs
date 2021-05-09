use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub trait Event: Send + Sync + Any {
    fn get_type_name(&self) -> String {
        let mut str = type_name::<Self>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();
        str.push_str(" - ");
        str.push_str(self.get_debug_info().as_str());
        str
    }
    fn get_debug_info(&self) -> String;
    fn redo(&self, events_rw: &mut EventsRw);
    fn undo(&self, events_rw: &mut EventsRw);
    fn as_boxed(&self) -> Box<dyn Event>;
}

type FrameEvent = (u64, Arc<dyn Event>);

pub struct Events {
    frame: u64,
    map: HashMap<TypeId, Vec<FrameEvent>>,
    history_map: HashMap<TypeId, Vec<FrameEvent>>,
}

impl Default for Events {
    fn default() -> Self {
        Self {
            frame: 0,
            map: HashMap::new(),
            history_map: HashMap::new(),
        }
    }
}

impl Events {
    pub fn send_event_from_history<T>(&mut self, event: T)
    where
        T: Event + 'static + Sized,
    {
        if let Some(list) = self.history_map.get_mut(&TypeId::of::<T>()) {
            list.push((self.frame + 1, Arc::new(event)));
        } else {
            self.history_map.insert(TypeId::of::<T>(), Vec::new());
            self.send_event_from_history(event);
        }
    }

    pub fn send_event<T>(&mut self, event: T)
    where
        T: Event + 'static + Sized,
    {
        if let Some(list) = self.map.get_mut(&TypeId::of::<T>()) {
            list.push((self.frame + 1, Arc::new(event)));
        } else {
            self.map.insert(TypeId::of::<T>(), Vec::new());
            self.send_event(event);
        }
    }

    fn read_events<T>(&self) -> Option<Vec<&T>>
    where
        T: Event + 'static,
    {
        if let Some(list) = self.map.get(&TypeId::of::<T>()) {
            let map = |i: &(u64, Arc<dyn Event>)| unsafe {
                &*Arc::into_raw(std::mem::transmute_copy(&i.1))
            };
            Some(list.iter().filter(|i| i.0 == self.frame).map(map).collect())
        } else {
            None
        }
    }

    fn read_history_events<T>(&self) -> Option<Vec<&T>>
    where
        T: Event + 'static,
    {
        if let Some(list) = self.history_map.get(&TypeId::of::<T>()) {
            let map = |i: &(u64, Arc<dyn Event>)| unsafe {
                &*Arc::into_raw(std::mem::transmute_copy(&i.1))
            };
            Some(list.iter().filter(|i| i.0 == self.frame).map(map).collect())
        } else {
            None
        }
    }

    pub fn read_all_events<T>(&self) -> Option<Vec<&T>>
    where
        T: Event + 'static,
    {
        let map_events = self.read_events();
        let history_map_events = self.read_history_events();
        if let Some(events) = map_events {
            if let Some(history_events) = history_map_events {
                return Some([events.as_slice(), history_events.as_slice()].concat());
            } else {
                return Some(events);
            }
        } else if history_map_events.is_some() {
            return history_map_events;
        }
        None
    }

    pub fn get_events_of_type(&self, type_filter: &[TypeId]) -> Vec<&dyn Event> {
        let mut events = Vec::new();
        self.map.iter().for_each(|(id, events_list)| {
            if type_filter.contains(id) {
                let map = |i: &(u64, Arc<dyn Event>)| unsafe {
                    &*Arc::into_raw(std::mem::transmute_copy(&i.1))
                };
                let mut list: Vec<&dyn Event> = events_list
                    .iter()
                    .filter(|i| i.0 == self.frame)
                    .map(map)
                    .collect();
                events.append(&mut list);
            }
        });
        events
    }

    pub fn update(&mut self, frame_count: u64) {
        for (_id, map) in self.map.iter_mut() {
            map.retain(|(frame, _el)| *frame >= frame_count);
        }
        for (_id, map) in self.history_map.iter_mut() {
            map.retain(|(frame, _el)| *frame >= frame_count);
        }
        self.frame = frame_count;
    }
}

pub type EventsRw = Arc<RwLock<Events>>;
