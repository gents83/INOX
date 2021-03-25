use std::any::Any;

use nrg_platform::*;

pub trait Command
where
    Self: Send + Sync + Any,
{
    fn execute(&mut self, events_rw: &mut EventsRw);
    fn undo(&mut self, events_rw: &mut EventsRw);
    fn box_clone(&self) -> Box<dyn Command>;
    fn get_type_name(&self) -> String {
        let v: Vec<&str> = std::any::type_name::<Self>().split(':').collect();
        let mut string = v.last().unwrap().to_string();
        string.push_str(self.get_debug_info().as_str());
        string
    }
    fn get_debug_info(&self) -> String;
}

impl Clone for Box<dyn Command> {
    fn clone(&self) -> Box<dyn Command> {
        self.box_clone()
    }
}

pub struct ExecuteCommand {
    command: Box<dyn Command>,
}

impl Event for ExecuteCommand {}

impl ExecuteCommand {
    pub fn new<C>(command: C) -> Self
    where
        C: Command,
    {
        Self {
            command: Box::new(command),
        }
    }

    pub fn get_command(&self) -> Box<dyn Command> {
        self.command.clone()
    }

    pub fn get_type_name(&self) -> String {
        self.command.as_ref().get_type_name()
    }
}
