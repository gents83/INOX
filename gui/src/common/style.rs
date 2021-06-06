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
    Default,
    DefaultCanvas,
    DefaultBackground,
    DefaultBorder,
    DefaultText,
    DefaultTitleBar,
    DefaultButton,
    FullActive,
    FullInactive,
    FullHighlight,
    Invisible,
}

impl WidgetStyle {
    #[inline]
    pub fn color(style: WidgetStyle) -> Vector4 {
        match style {
            Self::Default => hex_to_rgba(COLOR_BLUE_GRAY),
            Self::DefaultCanvas => hex_to_rgba(COLOR_BLACK),
            Self::DefaultBackground => hex_to_rgba(COLOR_ENGRAY),
            Self::DefaultBorder => hex_to_rgba(COLOR_SECONDARY),
            Self::DefaultText => hex_to_rgba(COLOR_WHITE),
            Self::DefaultTitleBar => hex_to_rgba(COLOR_BLUE),
            Self::DefaultButton => hex_to_rgba(COLOR_LIGHT_BLUE),
            Self::FullActive => hex_to_rgba(COLOR_WHITE),
            Self::FullInactive => hex_to_rgba(COLOR_GRAY),
            Self::FullHighlight => hex_to_rgba(COLOR_YELLOW),
            Self::Invisible => COLOR_TRANSPARENT.into(),
        }
    }
}
