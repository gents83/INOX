use std::{
    net::TcpListener,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use nrg_core::System;
use nrg_messenger::{MessageChannel, MessengerRw};
use nrg_profiler::debug_log;
use nrg_resources::ConfigBase;
use nrg_serialize::SerializeFile;

use crate::config::Config;

const SERVER_THREAD_NAME: &str = "Server Thread";

#[derive(Default)]
struct ConnectorData {
    can_continue: Arc<AtomicBool>,
}

pub struct Connector {
    _global_messenger: MessengerRw,
    _message_channel: MessageChannel,
    can_continue: Arc<AtomicBool>,
    host_address_and_port: String,
    server_thread: Option<JoinHandle<()>>,
}

impl Connector {
    pub fn new(global_messenger: &MessengerRw) -> Self {
        let _message_channel = MessageChannel::default();
        Self {
            _global_messenger: global_messenger.clone(),
            _message_channel,
            can_continue: Arc::new(AtomicBool::new(false)),
            host_address_and_port: String::new(),
            server_thread: None,
        }
    }
}

impl System for Connector {
    fn read_config(&mut self, plugin_name: &str) {
        let mut config = Config::default();
        config.load_from_file(config.get_filepath(plugin_name).as_path());

        self.host_address_and_port = config.host_address + ":" + config.port.to_string().as_str();
    }
    fn should_run_when_not_focused(&self) -> bool {
        false
    }

    fn init(&mut self) {
        if self.server_thread.is_none() {
            if let Ok(tcp_listener) = TcpListener::bind(self.host_address_and_port.as_str()) {
                self.can_continue.store(true, Ordering::SeqCst);
                let connector_data = ConnectorData {
                    can_continue: self.can_continue.clone(),
                };
                tcp_listener
                    .set_nonblocking(true)
                    .expect("Impossible to set non-blocking connection!!!");
                let builder = thread::Builder::new().name(SERVER_THREAD_NAME.to_string());
                let server_thread = builder
                    .spawn(move || {
                        while connector_data.can_continue.load(Ordering::SeqCst) {
                            for stream in tcp_listener.incoming().flatten() {
                                println!("Connection succeded: {}", stream.peer_addr().unwrap());
                            }
                        }
                    })
                    .unwrap();
                self.server_thread = Some(server_thread);
            } else {
                debug_log(
                    format!(
                        "Unable to bind to requested address {:?}",
                        self.host_address_and_port,
                    )
                    .as_str(),
                );
            }
        }
    }

    fn run(&mut self) -> bool {
        true
    }
    fn uninit(&mut self) {
        self.can_continue.store(false, Ordering::SeqCst);
    }
}
