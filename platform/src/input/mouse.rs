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
impl Event for MouseEvent {}

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
