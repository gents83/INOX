use super::command::*;

#[derive(Clone)]
struct TestCommand {
    text: String,
}

impl Command for TestCommand {
    fn execute(&mut self) {
        println!("Pre {}", self.text);
        println!("Executing TestCommand");
        self.text += "A";
        println!("Post {}", self.text);
    }

    fn undo(&mut self) {
        println!("Pre {}", self.text);
        println!("Undo TestCommand");
        self.text.pop();
        println!("Post {}", self.text);
    }

    fn box_clone(&self) -> Box<dyn Command> {
        Box::new((*self).clone())
    }
}

impl Default for TestCommand {
    fn default() -> Self {
        Self {
            text: String::new(),
        }
    }
}

#[test]
fn test_history() {
    use super::history::*;
    use nrg_platform::*;

    let events_rw = EventsRw::default();
    let mut history = CommandsHistory::default();
    history.set_events(events_rw.clone());

    {
        let mut events = events_rw.write().unwrap();
        let add_command_event = ExecuteCommand::new(TestCommand::default());
        events.send_event::<ExecuteCommand>(add_command_event);
    }

    {
        history.process_events();
        history.redo_last_command();
        history.undo_last_command();
        history.undo_last_command();
        history.redo_last_command();
    }
}
