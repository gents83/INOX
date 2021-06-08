use nrg_math::Vector4;
use nrg_serialize::{Deserialize, Serialize};

use crate::{colors::*, hex_to_rgba};

#[derive(Clone, Copy)]
pub enum WidgetInteractiveState {
    Inactive,
    Active,
    Hover,
    Pressed,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "nrg_serialize")]
pub enum WidgetStyle {
    DefaultBackground,
    DefaultCanvas,
    DefaultBorder,
    DefaultTitleBar,
    Default,
    DefaultText,
    DefaultButton,
    Invisible,
}

impl WidgetStyle {
    #[inline]
    pub fn color(style: WidgetStyle) -> Vector4 {
        match style {
            Self::DefaultBackground => hex_to_rgba("#15202B"),
            Self::DefaultCanvas => hex_to_rgba("#192734"),
            Self::DefaultBorder => hex_to_rgba("#22303C"),
            Self::DefaultTitleBar => hex_to_rgba("#0086B3"),
            Self::Default => hex_to_rgba("#00394D"),
            Self::DefaultText => hex_to_rgba("#FFFFFF"),
            Self::DefaultButton => hex_to_rgba("#006080"),
            Self::Invisible => COLOR_TRANSPARENT.into(),
        }
    }
}
