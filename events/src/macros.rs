#[macro_export]
macro_rules! implement_event {
    ($Type:ident) => {
        impl Event for $Type {
            fn as_boxed(&self) -> Box<dyn Event> {
                Box::new(self.clone())
            }
            fn redo(&self, events_rw: &mut EventsRw) {
                let mut events = events_rw.write().unwrap();
                events.send_event_from_history::<$Type>(*self);
            }
            fn undo(&self, _events_rw: &mut EventsRw) {
                eprintln!("Undo not implemented for {}", self.get_type_name().as_str());
            }
            fn get_debug_info(&self) -> String {
                "".to_string()
            }
        }
    };
}

#[macro_export]
macro_rules! implement_undoable_event {
    ($Type:ident, $func: ident, $debug_func: ident) => {
        impl Event for $Type {
            fn as_boxed(&self) -> Box<dyn Event> {
                Box::new(self.clone())
            }
            fn redo(&self, events_rw: &mut EventsRw) {
                let mut events = events_rw.write().unwrap();
                events.send_event_from_history(*self);
            }
            fn undo(&self, events_rw: &mut EventsRw) {
                let mut events = events_rw.write().unwrap();
                let event_to_send = $func(self);
                events.send_event_from_history(event_to_send);
            }
            fn get_debug_info(&self) -> String {
                $debug_func(self)
            }
        }
    };
}
