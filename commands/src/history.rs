use super::command::*;
use nrg_platform::*;

pub struct CommandsHistory {
    events_rw: EventsRw,
    commands: Vec<Box<dyn Command>>,
}

impl Default for CommandsHistory {
    fn default() -> Self {
        Self {
            events_rw: EventsRw::default(),
            commands: Vec::new(),
        }
    }
}

impl CommandsHistory {
    pub fn set_events(&mut self, events_rw: EventsRw) {
        self.events_rw = events_rw;
    }

    fn execute_operations(&mut self) {
        let events = self.events_rw.read().unwrap();
        if let Some(mut command_events) = events.read_events::<ExecuteCommand>() {
            for event in command_events.iter_mut() {
                let mut command = event.get_command();
                command.as_mut().execute();
                self.commands.push(command);
            }
        }
    }

    pub fn undo_last_command(&mut self) {
        if let Some(mut last_command) = self.commands.pop() {
            last_command.as_mut().undo();
        }
    }

    pub fn redo_last_command(&mut self) {
        if let Some(last_command) = self.commands.last() {
            let mut new_command = last_command.clone();
            new_command.as_mut().execute();
            self.commands.push(new_command);
        }
    }

    pub fn process_events(&mut self) {
        self.execute_operations();
    }
}
