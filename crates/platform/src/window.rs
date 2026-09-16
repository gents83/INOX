use std::path::Path;

use crate::{handle::*, KeyEvent, KeyTextEvent, MouseButton, MouseEvent, MouseState};
use inox_commands::CommandParser;
use inox_messenger::{implement_message, Listener, MessageHubRc};

pub const DEFAULT_DPI: f32 = 96.0;

/// Converts CSS-space pointer coordinates into device-pixel coordinates.
///
/// Browser layout dimensions may be zero while a canvas is hidden or before its
/// first layout pass. Keep the conversion finite and clamp the normalized
/// coordinates so input outside the canvas cannot produce invalid render input.
#[inline]
pub fn map_canvas_mouse_coordinates(
    client_x: f64,
    client_y: f64,
    left: f64,
    top: f64,
    css_width: f64,
    css_height: f64,
    pixel_width: u32,
    pixel_height: u32,
) -> (f64, f64, f32, f32) {
    let css_width = if css_width.is_finite() && css_width > 0.0 {
        css_width
    } else {
        1.0
    };
    let css_height = if css_height.is_finite() && css_height > 0.0 {
        css_height
    } else {
        1.0
    };
    let normalized_x = if client_x.is_finite() && left.is_finite() {
        ((client_x - left) / css_width).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let normalized_y = if client_y.is_finite() && top.is_finite() {
        ((client_y - top) / css_height).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (
        normalized_x * pixel_width.max(1) as f64,
        normalized_y * pixel_height.max(1) as f64,
        normalized_x as f32,
        normalized_y as f32,
    )
}

/// Returns the button associated with a browser mouse event.
///
/// Movement events do not represent a button press, even though browsers
/// report a default button index for them.
#[inline]
pub fn canvas_mouse_button(state: MouseState, button: i16) -> MouseButton {
    if state == MouseState::Move {
        MouseButton::None
    } else {
        match button {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            index => MouseButton::Other(u16::try_from(index).unwrap_or_default()),
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub enum WindowEvent {
    Show,
    Hide,
    Close,
    ScaleFactorChanged(f32),
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
            let values = command_parser.get_values_of("scale_factor");
            return Some(WindowEvent::ScaleFactorChanged(values[0]));
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
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    #[inline]
    pub fn x(&self) -> u32 {
        self.x
    }
    #[inline]
    pub fn y(&self) -> u32 {
        self.y
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn handle(&self) -> &Handle {
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
            WindowEvent::ScaleFactorChanged(v) => {
                scale_factor = *v;
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
