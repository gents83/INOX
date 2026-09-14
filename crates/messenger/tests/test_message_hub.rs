use std::sync::{Arc, Mutex};

use inox_commands::CommandParser;
use inox_messenger::{Listener, Message, MessageHub};

#[derive(Clone, Debug, PartialEq)]
struct TestMessage {
    value: u32,
}

impl Message for TestMessage {
    fn from_command_parser(_command_parser: CommandParser) -> Option<Self> {
        None
    }

    fn compare_and_discard(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

#[test]
fn listener_receives_flushed_messages() {
    let hub = Arc::new(MessageHub::default());
    let listener = Listener::new(&hub);
    listener.register::<TestMessage>();
    hub.send_event(TestMessage { value: 7 });
    hub.flush();

    let received = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&received);
    listener.process_messages::<TestMessage, _>(move |message| {
        captured.lock().unwrap().push(message.value);
    });

    assert_eq!(*received.lock().unwrap(), vec![7]);
    listener.unregister::<TestMessage>();
    hub.unregister_type::<TestMessage>();
}

#[test]
fn event_parser_extracts_serialized_event_sections() {
    assert_eq!(
        inox_messenger::get_events_from_string("before[[[one]]]middle[[[two]]]after".to_owned()),
        vec!["one", "two"]
    );
}
