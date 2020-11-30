use crate::viewport::*;
use crate::rasterizer::*;

pub struct Renderer {
    pub viewport: Viewport,
    pub scissors: Scissors,
    pub rasterizer: Rasterizer,
}


impl Default for Renderer {
    fn default() -> Renderer {
        Renderer {
            viewport: Viewport::default(),
            scissors: Scissors::default(),
            rasterizer: Rasterizer::default(),
        }
    }
}