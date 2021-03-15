use nrg_platform::*;
use nrg_serialize::*;

pub enum WidgetEvent {
    Entering(UID),
    Exiting(UID),
    Pressed(UID),
    Released(UID),
}
impl Event for WidgetEvent {}
pub struct GUI {}

impl GUI {
    pub fn register_widget_events(gui_events: &mut EventsRw) {
        let mut events = gui_events.write().unwrap();
        events.register_event::<WidgetEvent>();
    }
    pub fn unregister_widget_events(gui_events: &mut EventsRw) {
        let mut events = gui_events.write().unwrap();
        events.unregister_event::<WidgetEvent>();
    }
}
