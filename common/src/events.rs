use std::collections::HashMap;

pub type EventListenerID = u32;

pub trait EventListener {
    fn on_event_received(&self, event_type: u32);
}

pub trait Dispatcher {
    fn register_listener(&mut self, callback: impl FnMut(u32) + 'static) -> EventListenerID;
    fn notify(&self, event_type: u32);
}
pub struct EventDispatcher {
    pub callbacks: HashMap<EventListenerID, Box<dyn FnMut(u32)>>,
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self {
            callbacks: HashMap::new(),
        }
    }
}

impl Drop for EventDispatcher {
    fn drop(&mut self) {
        self.callbacks.clear();
    }
}

#[macro_export]
macro_rules! implement_dispatcher {
    ($ClassType:ty, $DispatcherFieldIdentifier:ident, $EventEnum:ident) => {  

        impl<'a> $ClassType {
            pub fn register_listener(&mut self, callback: impl FnMut(u32) + 'static) -> EventListenerID {
                unsafe { 
                    static mut identifier: EventListenerID = 0;
                    identifier += 1; 
                    self.$DispatcherFieldIdentifier.callbacks.insert(identifier, Box::new(callback));
                    identifier
                }
            }
            fn notify(&mut self, event_enum_type: $EventEnum) {
                for (_id, item) in self.$DispatcherFieldIdentifier.callbacks.iter_mut() {
                    (item)(event_enum_type as _);
                }
            }
        }

    }
}
