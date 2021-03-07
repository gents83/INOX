use nrg_math::*;

#[derive(Clone)]
pub struct WidgetMargins {
    pub left: f32,
    pub top: f32,
    pub bottom: f32,
    pub right: f32,
}
impl Default for WidgetMargins {
    fn default() -> Self {
        Self {
            top: 0.0,
            left: 0.0,
            right: 0.0,
            bottom: 0.0,
        }
    }
}

impl WidgetMargins {
    pub fn top_left(&self) -> Vector2f {
        Vector2f {
            x: self.left,
            y: self.top,
        }
    }
}

#[allow(dead_code)]
pub enum HorizontalAlignment {
    None,
    Left,
    Right,
    Center,
    Stretch,
}
#[allow(dead_code)]
pub enum VerticalAlignment {
    None,
    Top,
    Bottom,
    Center,
    Stretch,
}
