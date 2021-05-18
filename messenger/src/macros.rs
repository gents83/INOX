#[macro_export]
macro_rules! implement_message {
    ($Type:ident) => {
        impl $crate::Message for $Type {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_boxed(&self) -> Box<dyn $crate::Message> {
                Box::new(self.clone())
            }
            fn redo(&self, events_rw: &$crate::MessageBox) {
                let mut events = events_rw.write().unwrap();
                events.send(self.as_boxed()).ok();
            }
            fn undo(&self, _events_rw: &$crate::MessageBox) {
                eprintln!("Undo not implemented for {}", self.get_type_name().as_str());
            }
            fn get_debug_info(&self) -> String {
                "".to_string()
            }
        }
    };
}

#[macro_export]
macro_rules! implement_undoable_message {
    ($Type:ident, $func: ident, $debug_func: ident) => {
        impl $crate::Message for $Type {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_boxed(&self) -> Box<dyn $crate::Message> {
                Box::new(self.clone())
            }
            fn redo(&self, events_rw: &$crate::MessageBox) {
                let mut events = events_rw.write().unwrap();
                events.send(self.as_boxed()).ok();
            }
            fn undo(&self, events_rw: &$crate::MessageBox) {
                let mut events = events_rw.write().unwrap();
                let event_to_send = $func(self);
                events.send(event_to_send.as_boxed()).ok();
            }
            fn get_debug_info(&self) -> String {
                $debug_func(self)
            }
        }
    };
}
