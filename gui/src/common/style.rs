use super::colors::*;
use nrg_math::*;
use nrg_serialize::*;

#[derive(Clone, Copy)]
pub enum WidgetInteractiveState {
    Inactive,
    Active,
    Hover,
    Pressed,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub enum WidgetStyle {
    Default,
    DefaultCanvas,
    DefaultBackground,
    DefaultBorder,
    DefaultText,
    FullActive,
    FullInactive,
    FullHighlight,
}

impl WidgetStyle {
    pub fn color(style: &WidgetStyle, state: WidgetInteractiveState) -> Vector4f {
        match style {
            Self::Default => match state {
                WidgetInteractiveState::Inactive => COLOR_BLACK,
                WidgetInteractiveState::Active => COLOR_GRAY,
                WidgetInteractiveState::Hover => COLOR_LIGHT_GRAY,
                WidgetInteractiveState::Pressed => COLOR_GRAY,
            },
            Self::DefaultCanvas => match state {
                WidgetInteractiveState::Inactive => COLOR_BLACK,
                WidgetInteractiveState::Active => COLOR_BLACK,
                WidgetInteractiveState::Hover => COLOR_BLACK,
                WidgetInteractiveState::Pressed => COLOR_BLACK,
            },
            Self::DefaultBackground => match state {
                WidgetInteractiveState::Inactive => COLOR_DARKEST_GRAY,
                WidgetInteractiveState::Active => COLOR_DARKEST_GRAY,
                WidgetInteractiveState::Hover => COLOR_LIGHT_GRAY,
                WidgetInteractiveState::Pressed => COLOR_DARKEST_GRAY,
            },
            Self::DefaultBorder => match state {
                WidgetInteractiveState::Inactive => COLOR_BLACK,
                WidgetInteractiveState::Active => COLOR_DARK_GRAY,
                WidgetInteractiveState::Hover => COLOR_GRAY,
                WidgetInteractiveState::Pressed => COLOR_LIGHT_GRAY,
            },
            Self::DefaultText => match state {
                WidgetInteractiveState::Inactive => COLOR_LIGHT_GRAY,
                WidgetInteractiveState::Active => COLOR_WHITE,
                WidgetInteractiveState::Hover => COLOR_LIGHT_GRAY,
                WidgetInteractiveState::Pressed => COLOR_WHITE,
            },
            Self::FullActive => match state {
                WidgetInteractiveState::Inactive => COLOR_WHITE,
                WidgetInteractiveState::Active => COLOR_WHITE,
                WidgetInteractiveState::Hover => COLOR_WHITE,
                WidgetInteractiveState::Pressed => COLOR_WHITE,
            },
            Self::FullInactive => match state {
                WidgetInteractiveState::Inactive => COLOR_DARKEST_GRAY,
                WidgetInteractiveState::Active => COLOR_DARKEST_GRAY,
                WidgetInteractiveState::Hover => COLOR_DARKEST_GRAY,
                WidgetInteractiveState::Pressed => COLOR_DARKEST_GRAY,
            },
            Self::FullHighlight => match state {
                WidgetInteractiveState::Inactive => COLOR_YELLOW,
                WidgetInteractiveState::Active => COLOR_YELLOW,
                WidgetInteractiveState::Hover => COLOR_YELLOW,
                WidgetInteractiveState::Pressed => COLOR_YELLOW,
            },
        }
    }
}
