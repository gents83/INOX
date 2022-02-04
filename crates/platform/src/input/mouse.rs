use std::collections::HashMap;

use sabi_commands::CommandParser;
use sabi_messenger::implement_message;

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum MouseButton {
    None,
    Left,
    Right,
    Middle,
    Other(u16),
}
#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum MouseState {
    Invalid,
    Move,
    DoubleClick,
    Down,
    Up,
}

#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
pub struct MouseEvent {
    pub x: f64,
    pub y: f64,
    pub normalized_x: f32,
    pub normalized_y: f32,
    pub button: MouseButton,
    pub state: MouseState,
}
implement_message!(MouseEvent, mouse_event_from_command_parser);

impl MouseEvent {
    fn mouse_event_from_command_parser(command_parser: CommandParser) -> Option<Self> {
        if command_parser.has("mouse_move") {
            let values = command_parser.get_values_of("mouse_move");
            return Some(MouseEvent {
                x: values[0],
                y: values[1],
                normalized_x: values[0] as _,
                normalized_y: values[1] as _,
                button: MouseButton::None,
                state: MouseState::Move,
            });
        } else if command_parser.has("mouse_left_down") {
            let values = command_parser.get_values_of("mouse_left_down");
            return Some(MouseEvent {
                x: values[0],
                y: values[1],
                normalized_x: values[0] as _,
                normalized_y: values[1] as _,
                button: MouseButton::Left,
                state: MouseState::Down,
            });
        } else if command_parser.has("mouse_right_down") {
            let values = command_parser.get_values_of("mouse_right_down");
            return Some(MouseEvent {
                x: values[0],
                y: values[1],
                normalized_x: values[0] as _,
                normalized_y: values[1] as _,
                button: MouseButton::Right,
                state: MouseState::Down,
            });
        } else if command_parser.has("mouse_left_up") {
            let values = command_parser.get_values_of("mouse_left_up");
            return Some(MouseEvent {
                x: values[0],
                y: values[1],
                normalized_x: values[0] as _,
                normalized_y: values[1] as _,
                button: MouseButton::Left,
                state: MouseState::Up,
            });
        } else if command_parser.has("mouse_right_up") {
            let values = command_parser.get_values_of("mouse_right_up");
            return Some(MouseEvent {
                x: values[0],
                y: values[1],
                normalized_x: values[0] as _,
                normalized_y: values[1] as _,
                button: MouseButton::Right,
                state: MouseState::Up,
            });
        }
        None
    }
}

impl Default for MouseEvent {
    #[inline]
    fn default() -> Self {
        Self {
            x: 0.,
            y: 0.,
            normalized_x: 0.,
            normalized_y: 0.,
            button: MouseButton::None,
            state: MouseState::Move,
        }
    }
}

pub struct MouseData {
    pub(super) pos_x: f64,
    pub(super) pos_y: f64,
    pub(super) move_x: f64,
    pub(super) move_y: f64,
    pub(super) is_pressed: bool,
    pub(super) buttons: HashMap<MouseButton, MouseState>,
}

impl Default for MouseData {
    fn default() -> Self {
        Self {
            pos_x: 0.0,
            pos_y: 0.0,
            move_x: 0.0,
            move_y: 0.0,
            is_pressed: false,
            buttons: HashMap::new(),
        }
    }
}

impl MouseData {
    pub fn get_x(&self) -> f64 {
        self.pos_x
    }
    pub fn get_y(&self) -> f64 {
        self.pos_y
    }
    pub fn movement_x(&self) -> f64 {
        self.move_x
    }
    pub fn movement_y(&self) -> f64 {
        self.move_y
    }
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }
    pub fn get_button_state(&self, button: MouseButton) -> MouseState {
        if let Some(button) = self.buttons.get(&button) {
            *button
        } else {
            MouseState::Invalid
        }
    }
    pub fn is_button_down(&self, button: MouseButton) -> bool {
        if let Some(button) = self.buttons.get(&button) {
            *button == MouseState::Down
        } else {
            false
        }
    }
    pub fn is_button_up(&self, button: MouseButton) -> bool {
        if let Some(button) = self.buttons.get(&button) {
            *button == MouseState::Up
        } else {
            false
        }
    }
}
