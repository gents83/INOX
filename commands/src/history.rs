use super::command::*;
use nrg_platform::*;

enum CommandHistoryOperation {
    Undo,
    Redo,
}
pub struct CommandsHistory {
    events_rw: EventsRw,
    current_index: i32,
    commands: Vec<Box<dyn Command>>,
    operations: Vec<CommandHistoryOperation>,
}

impl CommandsHistory {
    pub fn new(events_rw: &EventsRw) -> Self {
        Self {
            events_rw: events_rw.clone(),
            current_index: -1,
            commands: Vec::new(),
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
                    if !self.commands.is_empty() && self.current_index < self.commands.len() as i32
                    {
                        let last_command = {
                            if self.current_index < 0 {
                                self.current_index = 0;
                                &mut self.commands[0]
                            } else {
                                &mut self.commands[self.current_index as usize]
                            }
                        };
                        last_command.as_mut().execute(&mut self.events_rw);
                        self.current_index = self.update_current_index(self.current_index + 1);
                    }
                }
                CommandHistoryOperation::Undo => {
                    if !self.commands.is_empty() && self.current_index >= 0 {
                        let last_command = {
                            if self.current_index >= self.commands.len() as i32 {
                                let last = self.commands.len() - 1;
                                &mut self.commands[last]
                            } else {
                                &mut self.commands[self.current_index as usize]
                            }
                        };
                        last_command.as_mut().undo(&mut self.events_rw);
                        self.current_index = self.update_current_index(self.current_index - 1);
                    }
                }
            }
        }
        self.operations.clear();
    }

    fn update_current_index(&self, index: i32) -> i32 {
        if self.commands.is_empty() || index < 0 {
            return -1;
        } else if index as usize >= self.commands.len() {
            return self.commands.len() as i32;
        }
        index as _
    }

    pub fn update(&mut self) {
        self.process_operations();
        let mut new_commands = self.process_events();
        for command in new_commands.iter_mut() {
            command.as_mut().execute(&mut self.events_rw);
        }
        if !new_commands.is_empty() {
            if self.current_index >= 0 {
                self.commands.truncate(self.current_index as usize + 1);
            } else {
                self.commands.clear();
            }
            self.commands.append(&mut new_commands);
            self.current_index = self.update_current_index(self.commands.len() as i32 - 1);
        }
    }

    pub fn get_current_index(&self) -> i32 {
        self.current_index
    }

    pub fn get_commands_history_as_string(&self) -> Option<Vec<String>> {
        if self.commands.is_empty() {
            None
        } else {
            let mut str: Vec<String> = Vec::new();
            for c in self.commands.iter() {
                str.push(c.get_type_name().to_string());
            }
            Some(str)
        }
    }
}
