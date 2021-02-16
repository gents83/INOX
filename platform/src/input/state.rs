#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum InputState {
    Invalid,
    Released,
    JustPressed,
    Pressed,
    JustReleased,
}
