use nrg_math::{Vector2i, Vector2u};
use nrg_serialize::{Deserialize, Serialize};

use crate::{Screen, WidgetDataGetter};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(crate = "nrg_serialize")]
pub enum ContainerFillType {
    None,
    Vertical,
    Horizontal,
}

pub trait ContainerTrait: WidgetDataGetter {
    fn apply_fit_to_content(&mut self) {
        let data = self.get_data_mut();
        let fill_type = data.state.get_fill_type();
        let keep_fixed_height = data.state.should_keep_fixed_height();
        let keep_fixed_width = data.state.should_keep_fixed_width();
        let space = data.state.get_space_between_elements();
        let use_space_before_after = data.state.should_use_space_before_and_after();

        let node = &mut data.node;
        let parent_pos = data.state.get_position();
        let parent_size = data.state.get_size();

        let mut children_min_pos: Vector2i = [i32::max_value(), i32::max_value()].into();
        let mut children_size: Vector2u = [0, 0].into();
        let mut index = 0;
        node.propagate_on_children_mut(|w| {
            let child_stroke =
                Screen::convert_size_into_pixels(w.get_data().graphics.get_stroke().into());
            let child_state = &mut w.get_data_mut().state;
            let child_pos = child_state.get_position();
            let child_size = child_state.get_size();
            children_min_pos.x = children_min_pos
                .x
                .min(child_pos.x as i32 - child_stroke.x as i32)
                .max(0);
            children_min_pos.y = children_min_pos
                .y
                .min(child_pos.y as i32 - child_stroke.y as i32)
                .max(0);
            match fill_type {
                ContainerFillType::Vertical => {
                    if (use_space_before_after && index == 0) || index > 0 {
                        children_size.y += space;
                    }
                    if !child_state.is_pressed() {
                        child_state
                            .set_position([child_pos.x, parent_pos.y + children_size.y].into());
                    }
                    children_size.y += child_size.y + child_stroke.y * 2;
                    children_size.x = children_size.x.max(child_size.x + child_stroke.x * 2);
                }
                ContainerFillType::Horizontal => {
                    if (use_space_before_after && index == 0) || index > 0 {
                        children_size.x += space;
                    }
                    if !child_state.is_pressed() {
                        child_state
                            .set_position([parent_pos.x + children_size.x, child_pos.y].into());
                    }
                    children_size.x += child_size.x + child_stroke.x * 2;
                    children_size.y = children_size.y.max(child_size.y + child_stroke.y * 2);
                }
                _ => {
                    children_size.x = parent_size.x;
                    children_size.y = parent_size.y;
                }
            }
            index += 1;
        });
        if use_space_before_after && fill_type == ContainerFillType::Vertical {
            children_size.y += space;
        }
        if use_space_before_after && fill_type == ContainerFillType::Horizontal {
            children_size.x += space;
        }
        if keep_fixed_width {
            children_size.x = parent_size.x;
        }
        if keep_fixed_height {
            children_size.y = parent_size.y;
        }
        data.state.set_size(children_size);
    }
}
