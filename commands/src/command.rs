use std::any::Any;

use nrg_platform::*;

pub trait Command
where
    Self: Send + Sync + Any,
{
    fn execute(&mut self);
    fn undo(&mut self);
    fn box_clone(&self) -> Box<dyn Command>;
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
}
