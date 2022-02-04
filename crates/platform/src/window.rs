use std::path::Path;

use crate::{handle::*, KeyEvent, KeyTextEvent, MouseEvent};
use inox_commands::CommandParser;
use inox_messenger::{implement_message, Listener, MessageHubRc};

pub const DEFAULT_DPI: f32 = 96.0;

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub enum WindowEvent {
    Show,
    Hide,
    Close,
    DpiChanged(f32, f32),
    SizeChanged(u32, u32),
    PosChanged(u32, u32),
    RequestChangeVisible(bool),
    RequestChangeTitle(String),
    RequestChangePos(u32, u32),
    RequestChangeSize(u32, u32),
}
implement_message!(
    WindowEvent,
    window_event_from_command_parser,
    compare_and_discard
);

impl WindowEvent {
    fn compare_and_discard(&self, _other: &Self) -> bool {
        false
    }
    fn window_event_from_command_parser(command_parser: CommandParser) -> Option<Self> {
        if command_parser.has("window_show") {
            return Some(WindowEvent::Show);
        } else if command_parser.has("window_hide") {
            return Some(WindowEvent::Hide);
        } else if command_parser.has("window_close") {
            return Some(WindowEvent::Close);
        } else if command_parser.has("dpi_changed") {
            let values = command_parser.get_values_of("dpi_changed");
            return Some(WindowEvent::DpiChanged(values[0], values[1]));
        } else if command_parser.has("window_size_changed") {
            let values = command_parser.get_values_of("window_size_changed");
            return Some(WindowEvent::SizeChanged(values[0], values[1]));
        } else if command_parser.has("window_position_changed") {
            let values = command_parser.get_values_of("window_position_changed");
            return Some(WindowEvent::PosChanged(values[0], values[1]));
        } else if command_parser.has("window_visible") {
            let values = command_parser.get_values_of("window_visible");
            return Some(WindowEvent::RequestChangeVisible(values[0]));
        } else if command_parser.has("window_title") {
            let values = command_parser.get_values_of::<String>("window_title");
            return Some(WindowEvent::RequestChangeTitle(values[0].clone()));
        } else if command_parser.has("window_size") {
            let values = command_parser.get_values_of("window_size");
            return Some(WindowEvent::RequestChangeSize(values[0], values[1]));
        } else if command_parser.has("window_position") {
            let values = command_parser.get_values_of("window_position");
            return Some(WindowEvent::RequestChangePos(values[0], values[1]));
        }
        None
    }
}

pub struct Window {
    handle: Handle,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    scale_factor: f32,
    listener: Listener,
    can_continue: bool,
}

unsafe impl Send for Window {}
unsafe impl Sync for Window {}

impl Window {
    pub fn create(
        title: String,
        x: u32,
        y: u32,
        mut width: u32,
        mut height: u32,
        icon_path: &Path,
        message_hub: &MessageHubRc,
    ) -> Self {
        message_hub
            .register_type::<WindowEvent>()
            .register_type::<KeyEvent>()
            .register_type::<KeyTextEvent>()
            .register_type::<MouseEvent>();

        let listener = Listener::new(message_hub);
        listener.register::<WindowEvent>();

        let mut scale_factor = 1.0;
        let handle = Window::create_handle(
            title,
            x,
            y,
            &mut width,
            &mut height,
            &mut scale_factor,
            icon_path,
            message_hub,
        );
        Self {
            handle,
            x,
            y,
            width,
            height,
            scale_factor,
            listener,
            can_continue: true,
        }
    }

    #[inline]
    pub fn get_scale_factor(&self) -> f32 {
        self.scale_factor
    }

    #[inline]
    pub fn get_x(&self) -> u32 {
        self.x
    }
    #[inline]
    pub fn get_y(&self) -> u32 {
        self.y
    }

    #[inline]
    pub fn get_width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn get_heigth(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn get_handle(&self) -> &Handle {
        &self.handle
    }

    #[inline]
    pub fn update(&mut self) -> bool {
        Window::internal_update(&self.handle);
        self.manage_window_events();
        self.can_continue
    }

    fn manage_window_events(&mut self) {
        let mut scale_factor = self.scale_factor;
        let mut can_continue = self.can_continue;
        let mut width = self.width;
        let mut height = self.height;
        let mut x = self.x;
        let mut y = self.y;

        self.listener.process_messages(|e: &WindowEvent| match e {
            WindowEvent::DpiChanged(new_x, _y) => {
                scale_factor = *new_x / DEFAULT_DPI;
            }
            WindowEvent::SizeChanged(new_width, new_height) => {
                width = *new_width;
                height = *new_height;
            }
            WindowEvent::PosChanged(new_x, new_y) => {
                x = *new_x;
                y = *new_y;
            }
            WindowEvent::Close => {
                can_continue = false;
            }
            WindowEvent::RequestChangeVisible(visible) => {
                Window::change_visibility(&self.handle, *visible);
            }
            WindowEvent::RequestChangeTitle(title) => {
                let mut title = title.clone();
                title.push('\0');
                Window::change_title(&self.handle, title.as_str());
            }
            WindowEvent::RequestChangePos(new_x, new_y) => {
                Window::change_position(&self.handle, *new_x, *new_y);
            }
            WindowEvent::RequestChangeSize(new_width, new_height) => {
                Window::change_size(&self.handle, *new_width, *new_height);
            }
            _ => {}
        });

        self.scale_factor = scale_factor;
        self.can_continue = can_continue;
        self.width = width;
        self.height = height;
        self.x = x;
        self.y = y;
    }
}
