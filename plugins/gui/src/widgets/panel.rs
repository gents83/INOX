use nrg_math::*;

const DEFAULT_WIDTH: f32 = 100.0;
const DEFAULT_HEIGHT: f32 = 75.0;

pub struct Panel {
    pos: Vector2f,
    size: Vector2f,
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            pos: Vector2f::default(),
            size: Vector2f::new(DEFAULT_WIDTH, DEFAULT_HEIGHT),
        }
    }
}

impl Panel {
    pub fn draw(&self) {}
}
