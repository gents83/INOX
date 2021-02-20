
pub enum PolygonModeType {
    Fill,
    Line,
    Point
}

pub enum CullingModeType {
    Disabled,
    Back,
    Front,
    Both
}

pub struct Rasterizer {
    pub depth_clamp_enabled: bool,
    pub depth_bias_enabled: bool,
    pub polygon_mode: PolygonModeType,
    pub culling_mode: CullingModeType,
    pub line_width: f32,
}

impl Default for Rasterizer {
    fn default() -> Rasterizer {
        Rasterizer {
            depth_clamp_enabled: false,
            depth_bias_enabled: false,
            polygon_mode: PolygonModeType::Fill,
            culling_mode: CullingModeType::Back,
            line_width: 1.0,
        }
    }
}
