use std::{
    io::Read,
    net::{SocketAddr, TcpListener, TcpStream},
    str::from_utf8,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use inox_core::{implement_unique_system_uid, ContextRc, System};
use inox_log::debug_log;
use inox_messenger::{Listener, MessageHubRc};
use inox_resources::{ConfigBase, ConfigEvent, SharedDataRc};
use inox_serialize::read_from_file;

use crate::config::Config;

const SERVER_THREAD_NAME: &str = "Server Thread";

#[derive(Default)]
struct ConnectorData {
    can_continue: Arc<AtomicBool>,
    message_hub: MessageHubRc,
    client_threads: Vec<JoinHandle<()>>,
}

pub struct Connector {
    config: Config,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    listener: Listener,
    can_continue: Arc<AtomicBool>,
    host_address_and_port: String,
    server_thread: Option<JoinHandle<()>>,
}

impl Connector {
    pub fn new(context: &ContextRc) -> Self {
        let listener = Listener::new(context.message_hub());
        Self {
            config: Config::default(),
            shared_data: context.shared_data().clone(),
            message_hub: context.message_hub().clone(),
            listener,
            can_continue: Arc::new(AtomicBool::new(false)),
            host_address_and_port: String::new(),
            server_thread: None,
        }
    }

    fn handle_events(&mut self) {
        self.listener
            .process_messages(|e: &ConfigEvent<Config>| match e {
                ConfigEvent::Loaded(filename, config) => {
                    if filename == self.config.get_filename() {
                        self.config = config.clone();
                        self.host_address_and_port = self.config.host_address.clone()
                            + ":"
                            + self.config.port.to_string().as_str();
                        println!("Host address and port: {}", self.host_address_and_port);
                    }
                }
            });
    }
}

implement_unique_system_uid!(Connector);

impl System for Connector {
    fn read_config(&mut self, plugin_name: &str) {
        self.listener.register::<ConfigEvent<Config>>();
        let message_hub = self.message_hub.clone();
        let filename = self.config.get_filename().to_string();
        read_from_file(
            self.config.get_filepath(plugin_name).as_path(),
            self.shared_data.serializable_registry(),
            Box::new(move |data: Config| {
                message_hub.send_event(ConfigEvent::Loaded(filename.clone(), data));
            }),
        );
    }
    fn should_run_when_not_focused(&self) -> bool {
        false
    }

    fn init(&mut self) {
        if self.server_thread.is_none() {
            if let Ok(tcp_listener) = TcpListener::bind(self.host_address_and_port.as_str()) {
                self.can_continue.store(true, Ordering::SeqCst);
                let mut connector_data = ConnectorData {
                    can_continue: self.can_continue.clone(),
                    message_hub: self.message_hub.clone(),
                    ..Default::default()
                };
                let builder = thread::Builder::new().name(SERVER_THREAD_NAME.to_string());
                let server_thread = builder
                    .spawn(move || {
                        while connector_data.can_continue.load(Ordering::SeqCst) {
                            match tcp_listener.accept() {
                                Ok((client_stream, addr)) => {
                                    let is_running = connector_data.can_continue.clone();
                                    let message_hub = connector_data.message_hub.clone();
                                    let thread = thread::Builder::new()
                                        .name("Reader".to_string())
                                        .spawn(move || {
                                            client_thread_execution(
                                                client_stream,
                                                addr,
                                                &message_hub,
                                                is_running,
                                            )
                                        })
                                        .unwrap();
                                    connector_data.client_threads.push(thread);
                                }
                                Err(e) => {
                                    println!("Connection failed: {}", e);
                                }
                            }
                        }
                    })
                    .unwrap();
                self.server_thread = Some(server_thread);
            } else {
                debug_log!(
                    "Unable to bind to requested address {:?}",
                    self.host_address_and_port,
                );
            }
        }
    }

    fn run(&mut self) -> bool {
        self.handle_events();

        true
    }
    fn uninit(&mut self) {
        self.can_continue.store(false, Ordering::SeqCst);

        self.listener.unregister::<ConfigEvent<Config>>();
    }
}

fn client_thread_execution(
    mut client_stream: TcpStream,
    addr: SocketAddr,
    message_hub: &MessageHubRc,
    is_running: Arc<AtomicBool>,
) {
    println!("Thread for client at {:?} started", addr);

    let mut buffer = [0u8; 1024];
    while is_running.load(Ordering::SeqCst) {
        match client_stream.read(&mut buffer) {
            Ok(_) => {
                let last = buffer
                    .iter()
                    .rposition(|&b| b != 0u8)
                    .unwrap_or(buffer.len());
                let s = String::from(from_utf8(&buffer).unwrap_or_default());
                let s = s.split_at(last + 1).0.to_string();

                message_hub.send_from_string(s);
            }
            Err(e) => {
                println!("[ServerThread] Failed to receive msg: {}", e);
            }
        }
    }

    println!("Thread for client at {:?} terminated", addr);
}
