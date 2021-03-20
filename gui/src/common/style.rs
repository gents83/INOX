use super::colors::*;
use nrg_math::*;

#[derive(Clone, Copy)]
pub enum WidgetInteractiveState {
    Inactive = 0,
    Active = 1,
    Hover = 2,
    Pressed = 3,
    Count = 4,
}

pub struct WidgetStyle {
    color: [Vector4f; WidgetInteractiveState::Count as _],
}

impl Default for WidgetStyle {
    fn default() -> Self {
        Self {
            color: [COLOR_BLACK, COLOR_GRAY, COLOR_LIGHT_GRAY, COLOR_GRAY],
        }
    }
}

impl WidgetStyle {
    pub fn get_color(&self, state: WidgetInteractiveState) -> Vector4f {
        self.color[state as usize]
    }
    pub fn default_background() -> Self {
        Self {
            color: [
                COLOR_BLACK,
                COLOR_DARKEST_GRAY,
                COLOR_LIGHT_GRAY,
                COLOR_DARKEST_GRAY,
            ],
        }
    }
    pub fn default_border() -> Self {
        Self {
            color: [COLOR_BLACK, COLOR_DARK_GRAY, COLOR_GRAY, COLOR_LIGHT_GRAY],
        }
    }
    pub fn default_text() -> Self {
        Self {
            color: [COLOR_LIGHT_GRAY, COLOR_WHITE, COLOR_LIGHT_GRAY, COLOR_WHITE],
        }
    }
    pub fn full_active() -> Self {
        Self {
            color: [COLOR_LIGHT_BLUE, COLOR_WHITE, COLOR_WHITE, COLOR_WHITE],
        }
    }
    pub fn full_inactive() -> Self {
        Self {
            color: [
                COLOR_DARK_GRAY,
                COLOR_DARK_GRAY,
                COLOR_DARK_GRAY,
                COLOR_DARK_GRAY,
            ],
        }
    }
}
