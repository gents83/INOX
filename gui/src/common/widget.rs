use std::any::TypeId;

use nrg_math::{Vector2, Vector4};
use nrg_messenger::{read_messages, Message};
use nrg_platform::{MouseEvent, MouseState};
use nrg_serialize::{typetag, Uid};

use crate::{
    add_space_before_after, add_widget_size, ContainerFillType, HorizontalAlignment, Screen,
    VerticalAlignment, WidgetDataGetter, WidgetEvent, WidgetInteractiveState,
};

pub const DEFAULT_LAYER_OFFSET: f32 = 0.1;
pub const DEFAULT_WIDGET_WIDTH: f32 = 12.;
pub const DEFAULT_WIDGET_HEIGHT: f32 = 12.;
pub const DEFAULT_WIDGET_SIZE: [f32; 2] = [DEFAULT_WIDGET_WIDTH, DEFAULT_WIDGET_HEIGHT];

#[typetag::serde(tag = "widget")]
pub trait Widget: BaseWidget + InternalWidget + Send + Sync {}

pub trait InternalWidget {
    fn widget_init(&mut self);
    fn widget_update(&mut self);
    fn widget_uninit(&mut self);
    fn widget_process_message(&mut self, _msg: &dyn Message);
}

pub trait BaseWidget: InternalWidget + WidgetDataGetter {
    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    fn init(&mut self) {
        let clip_area_in_px = Screen::get_draw_area();
        self.state_mut().set_drawing_area(clip_area_in_px);
        self.graphics_mut().init("UI");

        self.widget_init();

        if self.is_initialized() {
            self.node_mut().propagate_on_children_mut(|w| w.init());
        }

        self.move_to_layer(self.graphics().get_layer());
        self.invalidate_layout();
        self.mark_as_initialized();
    }
    fn update(&mut self, drawing_area_in_px: Vector4) {
        self.state_mut().set_drawing_area(drawing_area_in_px);
        self.process_messages();

        if self.state().is_dirty() {
            self.update_layout();
        }
        self.manage_style();

        let is_visible = self.graphics().is_visible();
        let filltype = self.state().get_fill_type();
        let space = self.state().get_space_between_elements() as f32;
        let use_space_before_after = self.state().should_use_space_before_and_after();
        let mut widget_clip = self.compute_children_drawing_area();
        if use_space_before_after {
            widget_clip = add_space_before_after(widget_clip, filltype, space);
        }
        self.node_mut().propagate_on_children_mut(|w| {
            if !is_visible && w.graphics().is_visible() {
                w.set_visible(is_visible);
            }
            w.update(widget_clip);
            widget_clip = add_widget_size(widget_clip, filltype, w);
            widget_clip = add_space_before_after(widget_clip, filltype, space);
        });

        {
            nrg_profiler::scoped_profile!("widget::wdget_update");
            self.widget_update();
        }

        self.graphics_mut().update();
    }
    fn uninit(&mut self) {
        self.node_mut().propagate_on_children_mut(|w| w.uninit());
        self.widget_uninit();
        self.graphics_mut().uninit();
    }
    fn process_messages(&mut self) {
        nrg_profiler::scoped_profile!("widget::process_messages");
        read_messages(self.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<MouseEvent>() {
                let e = msg.as_any().downcast_ref::<MouseEvent>().unwrap();
                self.manage_input(e);
            }
            if msg.type_id() == TypeId::of::<WidgetEvent>() {
                let e = msg.as_any().downcast_ref::<WidgetEvent>().unwrap();
                self.manage_events(e);
            }
            self.widget_process_message(msg);
        });
    }

    fn invalidate_layout(&mut self) {
        if self.node().has_parent() {
            let event = WidgetEvent::InvalidateLayout(self.node().get_parent());
            self.get_global_dispatcher()
                .write()
                .unwrap()
                .send(event.as_boxed())
                .ok();
        } else {
            self.mark_as_dirty();
        };
    }

    fn mark_as_dirty(&mut self) {
        self.state_mut().set_dirty(true);
        self.node_mut().propagate_on_children_mut(|w| {
            w.mark_as_dirty();
        });
    }

    fn id(&self) -> Uid {
        self.node().get_id()
    }
    fn set_position(&mut self, pos_in_px: Vector2) {
        if pos_in_px != self.state().get_position() {
            self.state_mut().set_position(pos_in_px);
            self.graphics_mut().set_position(pos_in_px);
            self.invalidate_layout();
        }
    }
    fn set_size(&mut self, size_in_px: Vector2) {
        if size_in_px != self.state().get_size() {
            self.state_mut().set_size(size_in_px);
            self.graphics_mut().set_size(size_in_px);
            self.invalidate_layout();
        }
    }

    fn compute_children_drawing_area(&self) -> Vector4 {
        let drawing_area = self.state().get_drawing_area();
        let pos = self.state().get_position();
        let size = self.state().get_size();
        let x = pos.x.max(drawing_area.x);
        let y = pos.y.max(drawing_area.y);
        [
            x,
            y,
            (size.x).min(drawing_area.z),
            (size.y).min(drawing_area.w),
        ]
        .into()
    }

    fn compute_offset_and_scale_from_alignment(&mut self) {
        nrg_profiler::scoped_profile!("widget::compute_offset_and_scale_from_alignment");

        let state = &self.state();

        let clip_rect = state.get_drawing_area();
        let clip_pos: Vector2 = [clip_rect.x, clip_rect.y].into();
        let clip_size: Vector2 = [clip_rect.z, clip_rect.w].into();

        let mut pos = state.get_position();
        let mut size = state.get_size();

        match state.get_horizontal_alignment() {
            HorizontalAlignment::Left => {
                pos.x = clip_pos.x;
            }
            HorizontalAlignment::Right => {
                pos.x = clip_pos.x + clip_size.x - size.x;
            }
            HorizontalAlignment::Center => {
                pos.x = clip_pos.x + clip_size.x / 2. - size.x / 2.;
            }
            HorizontalAlignment::Stretch => {
                pos.x = clip_pos.x;
                size.x = clip_size.x;
            }
            _ => {}
        }

        match state.get_vertical_alignment() {
            VerticalAlignment::Top => {
                pos.y = clip_pos.y;
            }
            VerticalAlignment::Bottom => {
                pos.y = clip_pos.y + clip_size.y - size.y;
            }
            VerticalAlignment::Center => {
                pos.y = clip_pos.y + clip_size.y / 2. - size.y / 2.;
            }
            VerticalAlignment::Stretch => {
                pos.y = clip_pos.y;
                size.y = clip_size.y;
            }
            _ => {}
        }

        size.x = size.x.min(clip_size.x);
        size.y = size.y.min(clip_size.y);
        pos.x = pos.x.max(clip_pos.x).min(clip_pos.x + clip_size.x - size.x);
        pos.y = pos.y.max(clip_pos.y).min(clip_pos.y + clip_size.y - size.y);

        self.set_position(pos);
        self.set_size(size);
    }

    fn apply_fit_to_content(&mut self) {
        nrg_profiler::scoped_profile!("widget::apply_fit_to_content");

        let fill_type = self.state().get_fill_type();
        let keep_fixed_height = self.state().should_keep_fixed_height();
        let keep_fixed_width = self.state().should_keep_fixed_width();
        let space = self.state().get_space_between_elements() as f32;
        let use_space_before_after = self.state().should_use_space_before_and_after();

        let parent_size = self.state().get_size();
        let node = &mut self.node_mut();

        let mut children_size: Vector2 = [0., 0.].into();
        let mut index = 0;
        node.propagate_on_children_mut(|w| {
            let child_state = &mut w.state_mut();
            let child_size = child_state.get_size();
            match fill_type {
                ContainerFillType::Vertical => {
                    if (use_space_before_after && index == 0) || index > 0 {
                        children_size.y += space;
                    }
                    children_size.y += child_size.y;
                    children_size.x = children_size.x.max(child_size.x);
                }
                ContainerFillType::Horizontal => {
                    if (use_space_before_after && index == 0) || index > 0 {
                        children_size.x += space;
                    }
                    children_size.x += child_size.x;
                    children_size.y = children_size.y.max(child_size.y);
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
        self.set_size(children_size);
    }
    fn manage_style(&mut self) {
        nrg_profiler::scoped_profile!("widget::manage_style");

        if self.state().is_hover() {
            let (color, border_color) = self.state().get_colors(WidgetInteractiveState::Hover);
            self.graphics_mut()
                .set_color(color)
                .set_border_color(border_color);
        } else if self.state().is_pressed() {
            let (color, border_color) = self.state().get_colors(WidgetInteractiveState::Pressed);
            self.graphics_mut()
                .set_color(color)
                .set_border_color(border_color);
        } else if self.state().is_active() {
            let (color, border_color) = self.state().get_colors(WidgetInteractiveState::Active);
            self.graphics_mut()
                .set_color(color)
                .set_border_color(border_color);
        } else {
            let (color, border_color) = self.state().get_colors(WidgetInteractiveState::Inactive);
            self.graphics_mut()
                .set_color(color)
                .set_border_color(border_color);
        }
    }
    fn manage_events(&mut self, event: &WidgetEvent) {
        nrg_profiler::scoped_profile!("widget::manage_events");
        if !self.graphics().is_visible() {
            return;
        }
        let id = self.id();
        match *event {
            WidgetEvent::InvalidateLayout(widget_id) => {
                if widget_id == id || self.node_mut().has_child(widget_id) {
                    self.invalidate_layout();
                }
            }
            WidgetEvent::Entering(widget_id) => {
                if widget_id == id && self.state().is_selectable() && self.state().is_active() {
                    self.state_mut().set_hover(true);
                }
            }
            WidgetEvent::Exiting(widget_id) => {
                if widget_id == id && self.state().is_selectable() && self.state().is_active() {
                    self.state_mut().set_hover(false);
                    self.state_mut().set_pressed(false);
                }
            }
            WidgetEvent::Released(widget_id, mouse_in_px) => {
                if widget_id == id && self.state().is_selectable() && self.state().is_active() {
                    self.state_mut().set_pressed(false);
                    if self.state().is_draggable() {
                        self.state_mut().set_dragging_position(mouse_in_px);
                    }
                }
            }
            WidgetEvent::Pressed(widget_id, mouse_in_px) => {
                if widget_id == id && self.state().is_selectable() && self.state().is_active() {
                    self.state_mut().set_pressed(true);
                    if self.state().is_draggable() {
                        self.state_mut().set_dragging_position(mouse_in_px);
                    }
                }
            }
            WidgetEvent::Dragging(widget_id, mouse_in_px) => {
                if widget_id == id
                    && self.state().is_draggable()
                    && self.state().is_selectable()
                    && self.state().is_active()
                {
                    self.state_mut()
                        .set_horizontal_alignment(HorizontalAlignment::None);
                    self.state_mut()
                        .set_vertical_alignment(VerticalAlignment::None);
                    let old_mouse_pos = self.state().get_dragging_position();
                    let offset = mouse_in_px - old_mouse_pos;
                    self.state_mut().set_dragging_position(mouse_in_px);
                    let current_pos = self.state().get_position();
                    self.set_position(current_pos + offset);
                }
            }
        }
    }
    fn manage_input(&mut self, event: &MouseEvent) {
        nrg_profiler::scoped_profile!("widget::manage_input");

        if !self.graphics().is_visible()
            || !self.state().is_active()
            || !self.state().is_selectable()
        {
            return;
        }
        let mut is_on_child = false;
        self.node().propagate_on_children(|w| {
            is_on_child |= w.state().is_hover();
        });
        if is_on_child {
            return;
        }

        let id = self.id();
        let mouse_in_px: Vector2 = [event.x as _, event.y as _].into();
        let is_inside = self.state().is_inside(mouse_in_px);

        let widget_event = if event.state == MouseState::Move {
            if is_inside && !self.state().is_hover() {
                Some(WidgetEvent::Entering(id))
            } else if !is_inside && self.state().is_hover() {
                Some(WidgetEvent::Exiting(id))
            } else if self.state().is_pressed() && self.state().is_draggable() {
                Some(WidgetEvent::Dragging(id, mouse_in_px))
            } else {
                None
            }
        } else if event.state == MouseState::Down && is_inside && !self.state().is_pressed() {
            Some(WidgetEvent::Pressed(id, mouse_in_px))
        } else if event.state == MouseState::Up && self.state().is_pressed() {
            Some(WidgetEvent::Released(id, mouse_in_px))
        } else {
            None
        };

        if let Some(event) = widget_event {
            self.get_global_dispatcher()
                .write()
                .unwrap()
                .send(event.as_boxed())
                .ok();
        }
    }
    fn move_to_layer(&mut self, layer: f32) {
        if (layer - self.graphics().get_layer()).abs() > f32::EPSILON {
            self.graphics_mut().set_layer(layer);
        }
    }

    fn update_layout(&mut self) {
        nrg_profiler::scoped_profile!("widget::update_layout");
        self.state_mut().set_dirty(false);
        self.apply_fit_to_content();
        self.compute_offset_and_scale_from_alignment();
        self.update_layers();
        if self.state().is_dirty() {
            self.invalidate_layout();
        }
    }

    fn update_layers(&mut self) {
        let layer = self.graphics().get_layer();
        self.node_mut().propagate_on_children_mut(|w| {
            w.move_to_layer(layer + DEFAULT_LAYER_OFFSET);
            w.update_layers();
        });
    }
    fn is_hover(&self) -> bool {
        let mut is_hover = self.state().is_hover();
        if !is_hover {
            self.node().propagate_on_children(|w| {
                if w.is_hover() {
                    is_hover = true;
                }
            });
        }
        is_hover
    }
    fn is_draggable(&self) -> bool {
        self.state().is_draggable()
    }
    fn is_selectable(&self) -> bool {
        self.state().is_selectable()
    }
    fn add_child(&mut self, widget: Box<dyn Widget>) -> Uid {
        let id = widget.id();
        self.node_mut().add_child(widget);

        self.invalidate_layout();
        id
    }
    fn set_visible(&mut self, visible: bool) {
        if self.graphics().is_visible() != visible {
            self.node_mut().propagate_on_children_mut(|w| {
                w.set_visible(visible);
            });
            self.graphics_mut().set_visible(visible);

            self.invalidate_layout();
        }
    }
}
