#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}
