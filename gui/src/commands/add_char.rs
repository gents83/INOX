use nrg_commands::Command;
use nrg_platform::{Event, EventsRw};
use nrg_serialize::UID;

use crate::TextEvent;

#[derive(Clone)]
pub struct AddCharCommand {
    widget_id: UID,
    character_index: i32,
    character: char,
}

impl AddCharCommand {
    pub fn new(widget_id: UID, character_index: i32, character: char) -> Self {
        Self {
            widget_id,
            character_index,
            character,
        }
    }
}

impl Event for AddCharCommand {}
impl Command for AddCharCommand {
    fn execute(&mut self, events_rw: &mut EventsRw) {
        let mut events = events_rw.write().unwrap();
        events.push_event_to_next_frame::<TextEvent>(TextEvent::AddChar(
            self.widget_id,
            self.character_index,
            self.character,
        ));
    }
    fn undo(&mut self, events_rw: &mut EventsRw) {
        let mut events = events_rw.write().unwrap();
        events.push_event_to_next_frame::<TextEvent>(TextEvent::RemoveChar(
            self.widget_id,
            self.character_index + 1,
            self.character,
        ));
    }
    fn box_clone(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
    fn get_debug_info(&self) -> String {
        format!("[{}]", self.character)
    }
}
