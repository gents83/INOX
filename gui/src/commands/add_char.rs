use nrg_commands::Command;
use nrg_platform::{Event, EventsRw};
use nrg_serialize::Uid;

use crate::TextEvent;

#[derive(Clone)]
pub struct AddCharCommand {
    widget_id: Uid,
    character_index: i32,
    character: char,
}

impl AddCharCommand {
    pub fn new(widget_id: Uid, character_index: i32, character: char) -> Self {
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
        events.send_event::<TextEvent>(TextEvent::AddChar(
            self.widget_id,
            self.character_index,
            self.character,
        ));
    }
    fn undo(&mut self, events_rw: &mut EventsRw) {
        let mut events = events_rw.write().unwrap();
        events.send_event::<TextEvent>(TextEvent::RemoveChar(
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
