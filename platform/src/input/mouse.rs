use std::collections::HashMap;

use crate::events::*;

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
    pub frame: u64,
    pub x: f64,
    pub y: f64,
    pub button: MouseButton,
    pub state: MouseState,
}
impl Event for MouseEvent {
    fn get_frame(&self) -> u64 {
        self.frame
    }
}

impl Default for MouseEvent {
    fn default() -> Self {
        Self {
            frame: 0,
            x: 0.0,
            y: 0.0,
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
    pub(super) is_dragging: bool,
    pub(super) buttons: HashMap<MouseButton, MouseState>,
}

impl Default for MouseData {
    fn default() -> Self {
        Self {
            pos_x: 0.0,
            pos_y: 0.0,
            move_x: 0.0,
            move_y: 0.0,
            is_dragging: false,
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
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
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
