use nrg_platform::EventsRw;

use crate::{Command, ExecuteCommand};

enum CommandHistoryOperation {
    Undo,
    Redo,
}
pub struct CommandsHistory {
    events_rw: EventsRw,
    undoable_commands: Vec<Box<dyn Command>>,
    redoable_commands: Vec<Box<dyn Command>>,
    operations: Vec<CommandHistoryOperation>,
}

impl CommandsHistory {
    pub fn new(events_rw: &EventsRw) -> Self {
        Self {
            events_rw: events_rw.clone(),
            undoable_commands: Vec::new(),
            redoable_commands: Vec::new(),
            operations: Vec::new(),
        }
    }

    fn process_events(&mut self) -> Vec<Box<dyn Command>> {
        let mut new_commands = Vec::new();
        let events = self.events_rw.read().unwrap();
        if let Some(mut command_events) = events.read_events::<ExecuteCommand>() {
            for event in command_events.iter_mut() {
                new_commands.push(event.get_command());
            }
        }
        new_commands
    }

    pub fn undo_last_command(&mut self) {
        self.operations.push(CommandHistoryOperation::Undo);
    }

    pub fn redo_last_command(&mut self) {
        self.operations.push(CommandHistoryOperation::Redo);
    }

    fn process_operations(&mut self) {
        for op in self.operations.iter() {
            match *op {
                CommandHistoryOperation::Redo => {
                    if !self.redoable_commands.is_empty() {
                        let mut last_command = self.redoable_commands.pop().unwrap();
                        last_command.as_mut().execute(&mut self.events_rw);
                        self.undoable_commands.push(last_command);
                    }
                }
                CommandHistoryOperation::Undo => {
                    if !self.undoable_commands.is_empty() {
                        let mut last_command = self.undoable_commands.pop().unwrap();
                        last_command.as_mut().undo(&mut self.events_rw);
                        self.redoable_commands.push(last_command);
                    }
                }
            }
        }
        self.operations.clear();
    }

    pub fn update(&mut self) {
        self.process_operations();
        let mut new_commands = self.process_events();
        for command in new_commands.iter_mut() {
            command.as_mut().execute(&mut self.events_rw);
        }
        if !new_commands.is_empty() {
            self.undoable_commands.append(&mut new_commands);
            self.redoable_commands.clear();
        }
    }

    pub fn get_undoable_commands_history_as_string(&self) -> Option<Vec<String>> {
        if self.undoable_commands.is_empty() {
            None
        } else {
            let mut str: Vec<String> = Vec::new();
            for c in self.undoable_commands.iter() {
                str.push(c.get_type_name().to_string());
            }
            Some(str)
        }
    }
    pub fn get_redoable_commands_history_as_string(&self) -> Option<Vec<String>> {
        if self.redoable_commands.is_empty() {
            None
        } else {
            let mut str: Vec<String> = Vec::new();
            for c in self.redoable_commands.iter() {
                str.push(c.get_type_name().to_string());
            }
            Some(str)
        }
    }

    pub fn clear(&mut self) -> &mut Self {
        self.redoable_commands.clear();
        self.undoable_commands.clear();
        self
    }
}
