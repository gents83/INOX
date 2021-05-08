use nrg_events::EventsRw;

use nrg_math::{Vector2, Vector4};
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
}

pub trait BaseWidget: InternalWidget + WidgetDataGetter {
    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    fn init(&mut self) {
        let clip_area_in_px = Screen::get_draw_area();
        self.get_data_mut().state.set_drawing_area(clip_area_in_px);
        self.get_data_mut().graphics.init("UI");

        self.widget_init();

        if self.is_initialized() {
            self.get_data_mut()
                .node
                .propagate_on_children_mut(|w| w.init());
        }

        self.update_layout();
        self.move_to_layer(self.get_data().graphics.get_layer());
        self.mark_as_initialized();
    }
    fn update(&mut self, drawing_area_in_px: Vector4) {
        self.get_data_mut()
            .state
            .set_drawing_area(drawing_area_in_px);
        self.update_layout();

        let is_visible = self.get_data().graphics.is_visible();
        let filltype = self.get_data().state.get_fill_type();
        let space = self.get_data().state.get_space_between_elements() as f32;
        let use_space_before_after = self.get_data().state.should_use_space_before_and_after();
        let mut widget_clip = self.compute_children_drawing_area();
        if use_space_before_after {
            widget_clip = add_space_before_after(widget_clip, filltype, space);
        }
        self.get_data_mut().node.propagate_on_children_mut(|w| {
            if !is_visible && w.get_data().graphics.is_visible() {
                w.set_visible(is_visible);
            }
            w.update(widget_clip);
            widget_clip = add_widget_size(widget_clip, filltype, w);
            widget_clip = add_space_before_after(widget_clip, filltype, space);
        });

        self.manage_input();
        self.manage_events();
        self.manage_style();

        self.widget_update();

        self.get_data_mut().graphics.update();
    }

    fn uninit(&mut self) {
        self.get_data_mut()
            .node
            .propagate_on_children_mut(|w| w.uninit());
        self.widget_uninit();
        self.get_data_mut().graphics.uninit();
    }

    fn id(&self) -> Uid {
        self.get_data().node.get_id()
    }
    fn set_position(&mut self, pos_in_px: Vector2) {
        if pos_in_px != self.get_data().state.get_position() {
            let data = self.get_data_mut();
            data.state.set_position(pos_in_px);
            data.graphics.set_position(pos_in_px);
        }
    }
    fn set_size(&mut self, size_in_px: Vector2) {
        if size_in_px != self.get_data().state.get_size() {
            let data = self.get_data_mut();
            data.state.set_size(size_in_px);
            data.graphics.set_size(size_in_px);
        }
    }

    fn compute_children_drawing_area(&self) -> Vector4 {
        let drawing_area = self.get_data().state.get_drawing_area();
        let pos = self.get_data().state.get_position();
        let size = self.get_data().state.get_size();
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
        let state = &self.get_data().state;

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
        let data = self.get_data_mut();
        let fill_type = data.state.get_fill_type();
        let keep_fixed_height = data.state.should_keep_fixed_height();
        let keep_fixed_width = data.state.should_keep_fixed_width();
        let space = data.state.get_space_between_elements() as f32;
        let use_space_before_after = data.state.should_use_space_before_and_after();

        let node = &mut data.node;
        let parent_size = data.state.get_size();

        let mut children_size: Vector2 = [0., 0.].into();
        let mut index = 0;
        node.propagate_on_children_mut(|w| {
            let child_state = &mut w.get_data_mut().state;
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
        let data = self.get_data_mut();

        if data.state.is_hover() {
            let (color, border_color) = data.state.get_colors(WidgetInteractiveState::Hover);
            data.graphics
                .set_color(color)
                .set_border_color(border_color);
        } else if data.state.is_pressed() {
            let (color, border_color) = data.state.get_colors(WidgetInteractiveState::Pressed);
            data.graphics
                .set_color(color)
                .set_border_color(border_color);
        } else if data.state.is_active() {
            let (color, border_color) = data.state.get_colors(WidgetInteractiveState::Active);
            data.graphics
                .set_color(color)
                .set_border_color(border_color);
        } else {
            let (color, border_color) = data.state.get_colors(WidgetInteractiveState::Inactive);
            data.graphics
                .set_color(color)
                .set_border_color(border_color);
        }
    }

    fn read_events(&self) -> Vec<WidgetEvent> {
        let mut my_events = Vec::new();
        let id = self.id();
        let read_data = self.get_shared_data().read().unwrap();
        let events_rw = &mut *read_data.get_unique_resource_mut::<EventsRw>();
        let events = events_rw.read().unwrap();
        if let Some(widget_events) = events.read_all_events::<WidgetEvent>() {
            for &event in widget_events.iter() {
                match &event {
                    WidgetEvent::Entering(widget_id) => {
                        if *widget_id == id {
                            my_events.push(*event);
                        }
                    }
                    WidgetEvent::Exiting(widget_id) => {
                        if *widget_id == id {
                            my_events.push(*event);
                        }
                    }
                    WidgetEvent::Released(widget_id, _mouse_in_px) => {
                        if *widget_id == id {
                            my_events.push(*event);
                        }
                    }
                    WidgetEvent::Pressed(widget_id, _mouse_in_px) => {
                        if *widget_id == id {
                            my_events.push(*event);
                        }
                    }
                    WidgetEvent::Dragging(widget_id, _mouse_in_px) => {
                        if *widget_id == id {
                            my_events.push(*event);
                        }
                    }
                }
            }
        }
        my_events
    }
    fn manage_events(&mut self) {
        let id = self.id();
        let events = self.read_events();
        for e in events.iter() {
            match e {
                WidgetEvent::Entering(widget_id) => {
                    let data = self.get_data_mut();
                    if *widget_id == id && data.state.is_selectable() {
                        data.state.set_hover(true);
                    }
                }
                WidgetEvent::Exiting(widget_id) => {
                    let data = self.get_data_mut();
                    if *widget_id == id && data.state.is_selectable() {
                        data.state.set_hover(false);
                        data.state.set_pressed(false);
                    }
                }
                WidgetEvent::Released(widget_id, mouse_in_px) => {
                    let data = self.get_data_mut();
                    if *widget_id == id && data.state.is_selectable() {
                        data.state.set_pressed(false);
                        if data.state.is_draggable() {
                            data.state.set_dragging_position(*mouse_in_px);
                        }
                    }
                }
                WidgetEvent::Pressed(widget_id, mouse_in_px) => {
                    let data = self.get_data_mut();
                    if *widget_id == id && data.state.is_selectable() {
                        data.state.set_pressed(true);
                        if data.state.is_draggable() {
                            data.state.set_dragging_position(*mouse_in_px);
                        }
                    }
                }
                WidgetEvent::Dragging(widget_id, mouse_in_px) => {
                    let data = self.get_data_mut();
                    if *widget_id == id && data.state.is_draggable() {
                        data.state
                            .set_horizontal_alignment(HorizontalAlignment::None);
                        data.state.set_vertical_alignment(VerticalAlignment::None);
                        let old_mouse_pos = data.state.get_dragging_position();
                        let offset = mouse_in_px - old_mouse_pos;
                        data.state.set_dragging_position(*mouse_in_px);
                        let current_pos = data.state.get_position();
                        self.set_position(current_pos + offset);
                    }
                }
            }
        }
    }

    fn manage_mouse_event(&mut self) -> Vec<WidgetEvent> {
        let mut widget_events = Vec::new();
        let id = self.id();
        let data = self.get_data();
        let read_data = self.get_shared_data().read().unwrap();
        let events_rw = &mut *read_data.get_unique_resource_mut::<EventsRw>();
        if let Some(mut mouse_events) = events_rw.read().unwrap().read_all_events::<MouseEvent>() {
            for event in mouse_events.iter_mut() {
                let mouse_in_px: Vector2 = [event.x as _, event.y as _].into();
                let is_inside = self.get_data().state.is_inside(mouse_in_px);

                if event.state == MouseState::Move {
                    if is_inside && !data.state.is_hover() {
                        widget_events.push(WidgetEvent::Entering(id));
                    } else if !is_inside && data.state.is_hover() {
                        widget_events.push(WidgetEvent::Exiting(id));
                    } else if data.state.is_pressed() && data.state.is_draggable() {
                        widget_events.push(WidgetEvent::Dragging(id, mouse_in_px));
                    }
                } else if event.state == MouseState::Down && is_inside && !data.state.is_pressed() {
                    widget_events.push(WidgetEvent::Pressed(id, mouse_in_px));
                } else if event.state == MouseState::Up && data.state.is_pressed() {
                    widget_events.push(WidgetEvent::Released(id, mouse_in_px));
                }
            }
        }
        widget_events
    }

    fn manage_input(&mut self) {
        let data = self.get_data_mut();
        if !data.graphics.is_visible() || !data.state.is_active() || !data.state.is_selectable() {
            return;
        }
        let mut is_on_child = false;
        data.node.propagate_on_children(|w| {
            is_on_child |= w.get_data().state.is_hover();
        });
        if is_on_child {
            return;
        }
        let widget_events = self.manage_mouse_event();
        let read_data = self.get_shared_data().read().unwrap();
        let events_rw = &mut *read_data.get_unique_resource_mut::<EventsRw>();
        for e in widget_events {
            let mut events = events_rw.write().unwrap();
            events.send_event(e);
        }
    }
    fn move_to_layer(&mut self, layer: f32) {
        if (layer - self.get_data().graphics.get_layer()).abs() > f32::EPSILON {
            let data = self.get_data_mut();
            data.graphics.set_layer(layer);
        }
    }

    fn update_layout(&mut self) {
        self.apply_fit_to_content();
        self.compute_offset_and_scale_from_alignment();
    }

    fn update_layers(&mut self) {
        let layer = self.get_data().graphics.get_layer();
        self.get_data_mut().node.propagate_on_children_mut(|w| {
            w.move_to_layer(layer + DEFAULT_LAYER_OFFSET);
            w.update_layers();
        });
    }
    fn is_hover(&self) -> bool {
        let mut is_hover = self.get_data().state.is_hover();
        if !is_hover {
            self.get_data().node.propagate_on_children(|w| {
                if w.is_hover() {
                    is_hover = true;
                }
            });
        }
        is_hover
    }
    fn is_draggable(&self) -> bool {
        self.get_data().state.is_draggable()
    }
    fn is_selectable(&self) -> bool {
        self.get_data().state.is_selectable()
    }
    fn add_child(&mut self, widget: Box<dyn Widget>) -> Uid {
        let id = widget.id();
        self.get_data_mut().node.add_child(widget);

        self.update_layout();
        self.update_layers();
        id
    }
    fn remove_children(&mut self) {
        self.get_data_mut().node.propagate_on_children_mut(|w| {
            w.get_data_mut().graphics.remove_meshes();
        });
        self.get_data_mut().node.remove_children();

        self.update_layout();
        self.update_layers();
    }
    fn has_child(&self, uid: Uid) -> bool {
        let mut found = false;
        self.get_data().node.propagate_on_children(|w| {
            if w.id() == uid {
                found = true;
            }
        });
        found
    }
    fn has_child_recursive(&self, uid: Uid) -> bool {
        let mut found = false;
        self.get_data().node.propagate_on_children(|w| {
            if w.id() == uid || w.has_child_recursive(uid) {
                found = true;
            }
        });
        found
    }
    fn set_visible(&mut self, visible: bool) {
        self.get_data_mut().node.propagate_on_children_mut(|w| {
            w.set_visible(visible);
        });
        self.get_data_mut().graphics.set_visible(visible);
    }
}
