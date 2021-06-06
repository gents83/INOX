use nrg_math::Vector4;
use nrg_serialize::{Deserialize, Serialize};

use crate::{colors::*, Screen};

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
    pub fn color(style: &WidgetStyle, state: WidgetInteractiveState) -> Vector4 {
        match style {
            Self::Default => match state {
                WidgetInteractiveState::Inactive => COLOR_BLACK.into(),
                WidgetInteractiveState::Active => COLOR_GRAY.into(),
                WidgetInteractiveState::Hover => COLOR_LIGHT_GRAY.into(),
                WidgetInteractiveState::Pressed => COLOR_GRAY.into(),
            },
            Self::DefaultCanvas => match state {
                WidgetInteractiveState::Inactive => COLOR_BLACK.into(),
                WidgetInteractiveState::Active => COLOR_BLACK.into(),
                WidgetInteractiveState::Hover => COLOR_BLACK.into(),
                WidgetInteractiveState::Pressed => COLOR_BLACK.into(),
            },
            Self::DefaultBackground => match state {
                WidgetInteractiveState::Inactive => COLOR_DARKEST_GRAY.into(),
                WidgetInteractiveState::Active => COLOR_DARKEST_GRAY.into(),
                WidgetInteractiveState::Hover => COLOR_LIGHT_GRAY.into(),
                WidgetInteractiveState::Pressed => COLOR_DARKEST_GRAY.into(),
            },
            Self::DefaultBorder => match state {
                WidgetInteractiveState::Inactive => COLOR_TRANSPARENT.into(),
                WidgetInteractiveState::Active => COLOR_TRANSPARENT.into(),
                WidgetInteractiveState::Hover => {
                    let mut border: Vector4 = COLOR_LIGHT_CYAN.into();
                    border.w = 10. * Screen::get_scale_factor();
                    border
                }
                WidgetInteractiveState::Pressed => {
                    let mut border: Vector4 = COLOR_WHITE.into();
                    border.w = 10. * Screen::get_scale_factor();
                    border
                }
            },
            Self::DefaultText => match state {
                WidgetInteractiveState::Inactive => COLOR_LIGHT_GRAY.into(),
                WidgetInteractiveState::Active => COLOR_WHITE.into(),
                WidgetInteractiveState::Hover => COLOR_LIGHT_GRAY.into(),
                WidgetInteractiveState::Pressed => COLOR_WHITE.into(),
            },
            Self::DefaultTitleBar => match state {
                WidgetInteractiveState::Inactive => COLOR_BLUE.into(),
                WidgetInteractiveState::Active => COLOR_LIGHT_BLUE.into(),
                WidgetInteractiveState::Hover => COLOR_LIGHT_BLUE.into(),
                WidgetInteractiveState::Pressed => COLOR_LIGHT_BLUE.into(),
            },
            Self::DefaultButton => match state {
                WidgetInteractiveState::Inactive => COLOR_GRAY.into(),
                WidgetInteractiveState::Active => COLOR_LIGHT_BLUE.into(),
                WidgetInteractiveState::Hover => COLOR_BLUE.into(),
                WidgetInteractiveState::Pressed => COLOR_LIGHT_CYAN.into(),
            },
            Self::FullActive => match state {
                WidgetInteractiveState::Inactive => COLOR_WHITE.into(),
                WidgetInteractiveState::Active => COLOR_WHITE.into(),
                WidgetInteractiveState::Hover => COLOR_WHITE.into(),
                WidgetInteractiveState::Pressed => COLOR_WHITE.into(),
            },
            Self::FullInactive => match state {
                WidgetInteractiveState::Inactive => COLOR_DARKEST_GRAY.into(),
                WidgetInteractiveState::Active => COLOR_DARKEST_GRAY.into(),
                WidgetInteractiveState::Hover => COLOR_DARKEST_GRAY.into(),
                WidgetInteractiveState::Pressed => COLOR_DARKEST_GRAY.into(),
            },
            Self::FullHighlight => match state {
                WidgetInteractiveState::Inactive => COLOR_YELLOW.into(),
                WidgetInteractiveState::Active => COLOR_YELLOW.into(),
                WidgetInteractiveState::Hover => COLOR_YELLOW.into(),
                WidgetInteractiveState::Pressed => COLOR_YELLOW.into(),
            },
            Self::Invisible => match state {
                WidgetInteractiveState::Inactive => COLOR_TRANSPARENT.into(),
                WidgetInteractiveState::Active => COLOR_TRANSPARENT.into(),
                WidgetInteractiveState::Hover => COLOR_TRANSPARENT.into(),
                WidgetInteractiveState::Pressed => COLOR_TRANSPARENT.into(),
            },
        }
    }
}
