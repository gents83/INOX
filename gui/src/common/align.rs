use nrg_math::Vector4;
use nrg_serialize::{Deserialize, Serialize};

use crate::{ContainerFillType, Widget};
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(crate = "nrg_serialize")]
pub enum HorizontalAlignment {
    None,
    Left,
    Right,
    Center,
    Stretch,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(crate = "nrg_serialize")]
pub enum VerticalAlignment {
    None,
    Top,
    Bottom,
    Center,
    Stretch,
}

pub fn add_space_before_after(
    mut widget_clip: Vector4,
    filltype: ContainerFillType,
    space: f32,
) -> Vector4 {
    match filltype {
        ContainerFillType::Horizontal => {
            widget_clip.x += space;
            widget_clip.z -= space;
            widget_clip.x = widget_clip.x.min(widget_clip.x + widget_clip.z);
        }
        ContainerFillType::Vertical => {
            widget_clip.y += space;
            widget_clip.w -= space;
            widget_clip.y = widget_clip.y.min(widget_clip.y + widget_clip.w);
        }
        _ => {}
    }
    widget_clip
}

pub fn add_widget_size(
    mut widget_clip: Vector4,
    filltype: ContainerFillType,
    widget: &dyn Widget,
) -> Vector4 {
    match filltype {
        ContainerFillType::Horizontal => {
            widget_clip.x += widget.state().get_size().x;
            widget_clip.z -= widget.state().get_size().x;
            widget_clip.x = widget_clip.x.min(widget_clip.x + widget_clip.z);
        }
        ContainerFillType::Vertical => {
            widget_clip.y += widget.state().get_size().y;
            widget_clip.w -= widget.state().get_size().y;
            widget_clip.y = widget_clip.y.min(widget_clip.y + widget_clip.w);
        }
        _ => {}
    }
    widget_clip
}
