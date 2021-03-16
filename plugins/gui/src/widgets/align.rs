use nrg_math::*;
pub const DEFAULT_WIDGET_SIZE: Vector2f = Vector2f { x: 32., y: 32. };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAlignment {
    None,
    Left,
    Right,
    Center,
    Stretch,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlignment {
    None,
    Top,
    Bottom,
    Center,
    Stretch,
}
