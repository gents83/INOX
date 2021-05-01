use std::collections::HashMap;

use nrg_events::{events::*, implement_event};

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
    pub button: MouseButton,
    pub state: MouseState,
}
implement_event!(MouseEvent);

impl Default for MouseEvent {
    fn default() -> Self {
        Self {
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
