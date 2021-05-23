use nrg_math::{VecBase, Vector2, Vector4};
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
            widget_clip.z = (widget_clip.z - space).max(0.);
            widget_clip.x = widget_clip.x.min(widget_clip.x + widget_clip.z);
        }
        ContainerFillType::Vertical => {
            widget_clip.y += space;
            widget_clip.w = (widget_clip.w - space).max(0.);
            widget_clip.y = widget_clip.y.min(widget_clip.y + widget_clip.w);
        }
        _ => {}
    }
    widget_clip
}

pub fn add_widget_size(
    mut widget_clip: Vector4,
    filltype: ContainerFillType,
    widget_index: usize,
    children: &[Box<dyn Widget>],
    space: f32,
    use_space_before_after: bool,
) -> Vector4 {
    if widget_index >= children.len() as _ {
        return widget_clip;
    }
    let widget = children[widget_index].as_ref();
    match filltype {
        ContainerFillType::Horizontal => {
            if widget.state().get_horizontal_alignment() == HorizontalAlignment::Stretch {
                let size = compute_next_children_space(
                    widget_index,
                    children,
                    space,
                    use_space_before_after,
                );
                widget_clip.z += size.x;
            }
            widget_clip.x += widget.state().get_size().x;
            widget_clip.z = (widget_clip.z - widget.state().get_size().x).max(0.);
            widget_clip.x = widget_clip.x.min(widget_clip.x + widget_clip.z);
        }
        ContainerFillType::Vertical => {
            if widget.state().get_vertical_alignment() == VerticalAlignment::Stretch {
                let size = compute_next_children_space(
                    widget_index,
                    children,
                    space,
                    use_space_before_after,
                );
                widget_clip.w += size.y;
            }
            widget_clip.y += widget.state().get_size().y;
            widget_clip.w = (widget_clip.w - widget.state().get_size().y).max(0.);
            widget_clip.y = widget_clip.y.min(widget_clip.y + widget_clip.w);
        }
        _ => {}
    }
    widget_clip
}

pub fn compute_child_clip_area(
    mut widget_clip: Vector4,
    filltype: ContainerFillType,
    widget_index: usize,
    children: &[Box<dyn Widget>],
    space: f32,
    use_space_before_after: bool,
) -> Vector4 {
    if widget_index >= children.len() as _ {
        return widget_clip;
    }
    let widget = children[widget_index].as_ref();
    match filltype {
        ContainerFillType::Horizontal => {
            if widget.state().get_horizontal_alignment() == HorizontalAlignment::Stretch {
                let size = compute_next_children_space(
                    widget_index,
                    children,
                    space,
                    use_space_before_after,
                );
                widget_clip.z -= size.x;
            }
        }
        ContainerFillType::Vertical => {
            if widget.state().get_vertical_alignment() == VerticalAlignment::Stretch {
                let size = compute_next_children_space(
                    widget_index,
                    children,
                    space,
                    use_space_before_after,
                );
                widget_clip.w -= size.y;
            }
        }
        _ => {}
    }
    widget_clip
}

fn compute_next_children_space(
    widget_index: usize,
    children: &[Box<dyn Widget>],
    space: f32,
    use_space_before_after: bool,
) -> Vector2 {
    let mut size = Vector2::default_zero();
    if widget_index + 1 < children.len() {
        (widget_index + 1..children.len()).for_each(|i| {
            size.x += space;
            size.x += children[i].as_ref().state().get_size().x;
            size.y += space;
            size.y += children[i].as_ref().state().get_size().y;
        });
        if use_space_before_after {
            size.y += space;
            size.x += space;
        }
    }
    size
}
