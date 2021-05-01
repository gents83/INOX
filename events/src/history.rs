use std::any::TypeId;

use crate::{Event, EventsRw};

enum EventsHistoryOperation {
    Undo,
    Redo,
}
pub struct EventsHistory {
    undoable_events: Vec<Box<dyn Event>>,
    redoable_events: Vec<Box<dyn Event>>,
    operations: Vec<EventsHistoryOperation>,
    registered_event_types: Vec<TypeId>,
}

impl Default for EventsHistory {
    fn default() -> Self {
        Self {
            undoable_events: Vec::new(),
            redoable_events: Vec::new(),
            operations: Vec::new(),
            registered_event_types: Vec::new(),
        }
    }
}

impl EventsHistory {
    fn process_operations(&mut self, events_rw: &mut EventsRw) {
        for op in self.operations.iter() {
            match *op {
                EventsHistoryOperation::Redo => {
                    if !self.redoable_events.is_empty() {
                        let mut last_event = self.redoable_events.pop().unwrap();
                        last_event.as_mut().redo(events_rw);
                        self.undoable_events.push(last_event);
                    }
                }
                EventsHistoryOperation::Undo => {
                    if !self.undoable_events.is_empty() {
                        let mut last_event = self.undoable_events.pop().unwrap();
                        last_event.as_mut().undo(events_rw);
                        self.redoable_events.push(last_event);
                    }
                }
            }
        }
        self.operations.clear();
    }

    fn process_events(&mut self, events_rw: &mut EventsRw) -> Vec<Box<dyn Event>> {
        let mut new_events = Vec::new();
        if self.registered_event_types.is_empty() {
            return new_events;
        }
        let events = events_rw.read().unwrap();
        let filtered_events = events.get_events_of_type(&self.registered_event_types);
        for event in filtered_events.iter() {
            let boxed_event = event.as_boxed();
            new_events.push(boxed_event);
        }
        new_events
    }

    pub fn register_event_as_undoable<T>(&mut self) -> &mut Self
    where
        T: Event,
    {
        let typeid = TypeId::of::<T>();
        if !self.registered_event_types.contains(&typeid) {
            self.registered_event_types.push(typeid);
        }
        self
    }

    pub fn undo_last_event(&mut self) {
        self.operations.push(EventsHistoryOperation::Undo);
    }

    pub fn redo_last_event(&mut self) {
        self.operations.push(EventsHistoryOperation::Redo);
    }

    pub fn update(&mut self, events_rw: &mut EventsRw) {
        let mut new_events = self.process_events(events_rw);
        if !new_events.is_empty() {
            self.undoable_events.append(&mut new_events);
        }
        self.process_operations(events_rw);
    }

    pub fn get_undoable_events_history_as_string(&self) -> Option<Vec<String>> {
        if self.undoable_events.is_empty() {
            None
        } else {
            let mut str: Vec<String> = Vec::new();
            for c in self.undoable_events.iter() {
                str.push(c.get_type_name());
            }
            Some(str)
        }
    }
    pub fn get_redoable_events_history_as_string(&self) -> Option<Vec<String>> {
        if self.redoable_events.is_empty() {
            None
        } else {
            let mut str: Vec<String> = Vec::new();
            for c in self.redoable_events.iter() {
                str.push(c.get_type_name());
            }
            Some(str)
        }
    }

    pub fn clear(&mut self) {
        self.redoable_events.clear();
        self.undoable_events.clear();
    }
}
