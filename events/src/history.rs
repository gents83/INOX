use std::any::TypeId;

use nrg_messenger::{Message, MessageBox, MessageChannel, MessengerRw};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventsHistoryOperation {
    Undo,
    Redo,
    Clear,
}
pub struct EventsHistory {
    undoable_events: Vec<Box<dyn Message>>,
    redoable_events: Vec<Box<dyn Message>>,
    operations: Vec<EventsHistoryOperation>,
    registered_event_types: Vec<TypeId>,
    events_dispatcher: Option<MessageBox>,
    message_channel: MessageChannel,
}

impl Default for EventsHistory {
    fn default() -> Self {
        Self {
            events_dispatcher: None,
            message_channel: MessageChannel::default(),
            undoable_events: Vec::new(),
            redoable_events: Vec::new(),
            operations: Vec::new(),
            registered_event_types: Vec::new(),
        }
    }
}

impl EventsHistory {
    fn process_operations(&mut self) {
        for op in self.operations.iter() {
            match *op {
                EventsHistoryOperation::Redo => {
                    if !self.redoable_events.is_empty() {
                        let mut last_event = self.redoable_events.pop().unwrap();
                        if let Some(events_rw) = &mut self.events_dispatcher {
                            last_event.as_mut().redo(events_rw);
                        }
                        self.undoable_events.push(last_event);
                    }
                }
                EventsHistoryOperation::Undo => {
                    if !self.undoable_events.is_empty() {
                        let mut last_event = self.undoable_events.pop().unwrap();
                        if let Some(events_rw) = &mut self.events_dispatcher {
                            last_event.as_mut().undo(events_rw);
                        }
                        self.redoable_events.push(last_event);
                    }
                }
                EventsHistoryOperation::Clear => {
                    self.redoable_events.clear();
                    self.undoable_events.clear();
                }
            }
        }
        self.operations.clear();
    }

    pub fn register_event_as_undoable<T>(&mut self, global_messenger: &MessengerRw) -> &mut Self
    where
        T: Message + Clone,
    {
        let typeid = TypeId::of::<T>();
        if !self.registered_event_types.contains(&typeid) {
            self.registered_event_types.push(typeid);

            if self.events_dispatcher.is_none() {
                self.events_dispatcher = Some(global_messenger.read().unwrap().get_dispatcher());
            }

            global_messenger.write().unwrap().register_type::<T>();
            global_messenger
                .write()
                .unwrap()
                .register_messagebox::<T>(self.message_channel.get_messagebox());
        }
        self
    }

    pub fn push(&mut self, operation: EventsHistoryOperation) {
        self.operations.push(operation);
    }

    pub fn update(&mut self) {
        while let Ok(msg) = self
            .message_channel
            .get_listener()
            .read()
            .unwrap()
            .try_recv()
        {
            self.undoable_events.push(msg);
        }
        self.process_operations();
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
}
