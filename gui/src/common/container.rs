use super::*;
use crate::screen::*;
use nrg_math::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerFillType {
    None,
    Vertical,
    Horizontal,
}

pub struct ContainerData {
    pub fill_type: ContainerFillType,
    pub fit_to_content: bool,
    pub space_between_elements: u32,
}

impl Default for ContainerData {
    fn default() -> Self {
        Self {
            fill_type: ContainerFillType::Vertical,
            fit_to_content: true,
            space_between_elements: 0,
        }
    }
}

pub trait ContainerTrait: WidgetDataGetter {
    fn get_container_data(&self) -> &ContainerData;
    fn get_container_data_mut(&mut self) -> &mut ContainerData;
    fn fill_type(&mut self, fill_type: ContainerFillType) -> &mut Self {
        self.get_container_data_mut().fill_type = fill_type;
        self
    }
    fn get_fill_type(&self) -> ContainerFillType {
        self.get_container_data().fill_type
    }
    fn fit_to_content(&mut self, fit_to_content: bool) -> &mut Self {
        self.get_container_data_mut().fit_to_content = fit_to_content;
        self
    }
    fn has_fit_to_content(&self) -> bool {
        self.get_container_data().fit_to_content
    }
    fn space_between_elements(&mut self, space_in_px: u32) -> &mut Self {
        self.get_container_data_mut().space_between_elements = space_in_px;
        self
    }
    fn get_space_between_elements(&self) -> u32 {
        self.get_container_data().space_between_elements
    }

    fn apply_fit_to_content(&mut self) {
        let fill_type = self.get_fill_type();
        let fit_to_content = self.has_fit_to_content();
        let space = self.get_space_between_elements();

        let data = self.get_data_mut();
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
                    children_size.y += space;
                    if !child_state.is_pressed() {
                        child_state
                            .set_position([child_pos.x, parent_pos.y + children_size.y].into());
                    }
                    children_size.y += child_size.y + child_stroke.y * 2;
                    children_size.x = children_size.x.max(child_size.x + child_stroke.x * 2);
                }
                ContainerFillType::Horizontal => {
                    children_size.x += space;
                    if !child_state.is_pressed() {
                        child_state
                            .set_position([parent_pos.x + children_size.x, child_pos.y].into());
                    }
                    children_size.x += child_size.x + child_stroke.x * 2;
                    children_size.y = children_size.y.max(child_size.y + child_stroke.y * 2);
                    if fit_to_content {
                    } else {
                        children_size.y = parent_size.y;
                    }
                }
                _ => {
                    children_size.x = parent_size.x;
                    children_size.y = parent_size.y;
                }
            }
            index += 1;
        });
        if !fit_to_content {
            children_size.x = parent_size.x;
            children_size.y = parent_size.y;
        }
        data.state.set_size(children_size);
    }
}
