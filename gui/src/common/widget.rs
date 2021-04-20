use nrg_graphics::Renderer;
use nrg_math::{VecBase, Vector2, Vector4};
use nrg_platform::{EventsRw, MouseEvent, MouseState};
use nrg_serialize::{typetag, Uid};

use crate::{
    ContainerFillType, HorizontalAlignment, Screen, VerticalAlignment, WidgetDataGetter,
    WidgetEvent, WidgetInteractiveState,
};

pub const DEFAULT_LAYER_OFFSET: f32 = 0.01;
pub const DEFAULT_WIDGET_WIDTH: f32 = 12.;
pub const DEFAULT_WIDGET_HEIGHT: f32 = 12.;
pub const DEFAULT_WIDGET_SIZE: [f32; 2] = [DEFAULT_WIDGET_WIDTH, DEFAULT_WIDGET_HEIGHT];

#[typetag::serde(tag = "widget")]
pub trait Widget: BaseWidget + InternalWidget + Send + Sync {}

pub trait InternalWidget {
    fn widget_init(&mut self, renderer: &mut Renderer);
    fn widget_update(&mut self, renderer: &mut Renderer, events: &mut EventsRw);
    fn widget_uninit(&mut self, renderer: &mut Renderer);
}

pub trait BaseWidget: InternalWidget + WidgetDataGetter {
    fn get_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    fn init(&mut self, renderer: &mut Renderer) {
        let clip_area_in_px = Screen::get_draw_area();
        self.get_data_mut().state.set_clip_area(clip_area_in_px);
        self.get_data_mut().graphics.init(renderer, "UI");

        self.widget_init(renderer);

        if self.is_initialized() {
            self.get_data_mut()
                .node
                .propagate_on_children_mut(|w| w.init(renderer));
        }

        self.move_to_layer(self.get_data().graphics.get_layer());
        self.update_layout();
        self.mark_as_initialized();
    }
    fn update(
        &mut self,
        drawing_area_in_px: Vector4,
        renderer: &mut Renderer,
        events_rw: &mut EventsRw,
    ) {
        self.get_data_mut().state.set_clip_area(drawing_area_in_px);

        let is_visible = self.get_data().state.is_visible();
        self.manage_input(events_rw);
        self.manage_events(events_rw);
        self.manage_style();

        self.widget_update(renderer, events_rw);

        self.update_layout();

        let widget_clip = self.compute_children_clip_area();
        self.get_data_mut().node.propagate_on_children_mut(|w| {
            if !is_visible && w.get_data().state.is_visible() {
                w.set_visible(is_visible);
            }
            w.update(widget_clip, renderer, events_rw);
        });

        self.get_data_mut().graphics.update(renderer, is_visible);
    }

    fn uninit(&mut self, renderer: &mut Renderer) {
        self.get_data_mut()
            .node
            .propagate_on_children_mut(|w| w.uninit(renderer));
        self.widget_uninit(renderer);
        self.get_data_mut().graphics.uninit(renderer);
    }

    fn id(&self) -> Uid {
        self.get_data().node.get_id()
    }
    fn set_position(&mut self, pos_in_px: Vector2) {
        if pos_in_px != self.get_data().state.get_position() {
            let data = self.get_data_mut();
            let current_pos = data.state.get_position();
            data.state.set_position(pos_in_px);
            let old_pos = Screen::convert_from_pixels_into_screen_space(current_pos);
            let new_pos = Screen::convert_from_pixels_into_screen_space(pos_in_px);
            data.graphics.translate(new_pos - old_pos);
        }
    }
    fn set_size(&mut self, size_in_px: Vector2) {
        if size_in_px != self.get_data().state.get_size() {
            let data = self.get_data_mut();
            let old_screen_scale = Screen::convert_size_from_pixels(data.state.get_size());
            let screen_size = Screen::convert_size_from_pixels(size_in_px);
            let scale = screen_size.div(old_screen_scale);
            data.state.set_size(size_in_px);
            let pos = Screen::convert_from_pixels_into_screen_space(data.state.get_position());
            data.graphics.translate(-pos);
            data.graphics.scale(scale);
            data.graphics.translate(pos);
        }
    }

    fn compute_children_clip_area(&self) -> Vector4 {
        let pos = self.get_data().state.get_position();
        let size = self.get_data().state.get_size();
        [pos.x, pos.y, pos.x + size.x, pos.y + size.y].into()
    }

    fn compute_offset_and_scale_from_alignment(&mut self) {
        let state = &self.get_data().state;

        let clip_rect = state.get_clip_area();
        let clip_min: Vector2 = [clip_rect.x, clip_rect.y].into();
        let clip_max: Vector2 = [clip_rect.z, clip_rect.w].into();

        let mut pos = state.get_position();
        let mut size = state.get_size();

        size.x = size.x.min((clip_max - clip_min).x);
        size.y = size.y.min((clip_max - clip_min).y);

        match state.get_horizontal_alignment() {
            HorizontalAlignment::Left => {
                pos.x = clip_min.x;
            }
            HorizontalAlignment::Right => {
                pos.x = clip_max.x - size.x;
            }
            HorizontalAlignment::Center => {
                pos.x = clip_min.x + (clip_max.x - clip_min.x) / 2. - size.x / 2.;
            }
            HorizontalAlignment::Stretch => {
                pos.x = clip_min.x;
                size.x = clip_max.x - clip_min.x;
            }
            _ => {}
        }

        match state.get_vertical_alignment() {
            VerticalAlignment::Top => {
                pos.y = clip_min.y;
            }
            VerticalAlignment::Bottom => {
                pos.y = clip_max.y - size.y;
            }
            VerticalAlignment::Center => {
                pos.y = clip_min.y + (clip_max.y - clip_min.y) / 2. - size.y / 2.;
            }
            VerticalAlignment::Stretch => {
                pos.y = clip_min.y;
                size.y = clip_max.y - clip_min.y;
            }
            _ => {}
        }
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
        let parent_pos = data.state.get_position();
        let parent_size = data.state.get_size();

        let mut children_min_pos: Vector2 = [f32::MAX, f32::MAX].into();
        let mut children_size: Vector2 = [0., 0.].into();
        let mut index = 0;
        node.propagate_on_children_mut(|w| {
            let child_state = &mut w.get_data_mut().state;
            let child_pos = child_state.get_position();
            let child_size = child_state.get_size();
            children_min_pos.x = children_min_pos.x.min(child_pos.x as _).max(0.);
            children_min_pos.y = children_min_pos.y.min(child_pos.y as _).max(0.);
            match fill_type {
                ContainerFillType::Vertical => {
                    if (use_space_before_after && index == 0) || index > 0 {
                        children_size.y += space;
                    }
                    w.set_position([child_pos.x, parent_pos.y + children_size.y].into());
                    children_size.y += child_size.y;
                    children_size.x = children_size.x.max(child_size.x);
                }
                ContainerFillType::Horizontal => {
                    if (use_space_before_after && index == 0) || index > 0 {
                        children_size.x += space;
                    }
                    w.set_position([parent_pos.x + children_size.x, child_pos.y].into());
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
    fn clip_in_area(&mut self) {
        let state = &self.get_data().state;

        let clip_rect = state.get_clip_area();
        let clip_min: Vector2 = [clip_rect.x, clip_rect.y].into();
        let clip_max: Vector2 = [clip_rect.z, clip_rect.w].into();

        let mut pos = Vector2::default_zero();
        pos.x = state.get_position().x as _;
        pos.y = state.get_position().y as _;
        let size = state.get_size();

        pos.x = pos.x.max(clip_min.x).min(clip_max.x - size.x).max(0.);
        pos.y = pos.y.max(clip_min.y).min(clip_max.y - size.y).max(0.);

        self.set_position([pos.x as _, pos.y as _].into());
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

    fn manage_events(&mut self, events: &mut EventsRw) {
        let id = self.id();
        let events = events.read().unwrap();
        if let Some(widget_events) = events.read_events::<WidgetEvent>() {
            for event in widget_events.iter() {
                match event {
                    WidgetEvent::Entering(widget_id) => {
                        let data = self.get_data_mut();
                        if *widget_id == id && data.state.is_selectable() {
                            data.state.set_hover(true);
                        } else {
                            data.state.set_hover(false);
                        }
                    }
                    WidgetEvent::Exiting(widget_id) => {
                        let data = self.get_data_mut();
                        if *widget_id == id && data.state.is_selectable() {
                            data.state.set_hover(false);
                            data.state.set_pressed(false);
                        }
                    }
                    WidgetEvent::Released(widget_id) => {
                        let data = self.get_data_mut();
                        if *widget_id == id && data.state.is_selectable() {
                            data.state.set_pressed(false);
                        }
                    }
                    WidgetEvent::Pressed(widget_id) => {
                        let data = self.get_data_mut();
                        if *widget_id == id && data.state.is_selectable() {
                            data.state.set_pressed(true);
                        } else {
                            data.state.set_pressed(false);
                        }
                    }
                    WidgetEvent::Dragging(widget_id, mouse_in_px) => {
                        if *widget_id == id && self.get_data().state.is_draggable() {
                            self.get_data_mut()
                                .state
                                .set_horizontal_alignment(HorizontalAlignment::None);
                            self.get_data_mut()
                                .state
                                .set_vertical_alignment(VerticalAlignment::None);

                            self.set_position(*mouse_in_px);
                        }
                    }
                }
            }
        }
    }

    fn manage_mouse_event(&mut self, event: &MouseEvent) -> Option<WidgetEvent> {
        let id = self.id();
        let data = self.get_data_mut();
        let mouse_in_px: Vector2 = [event.x as _, event.y as _].into();
        let is_inside = data.state.is_inside(mouse_in_px) /*&& data.graphics.is_inside(mouse_in_px)*/;

        if event.state == MouseState::Move {
            if is_inside && !data.state.is_hover() {
                return Some(WidgetEvent::Entering(id));
            } else if !is_inside && data.state.is_hover() {
                return Some(WidgetEvent::Exiting(id));
            } else if data.state.is_pressed() && data.state.is_draggable() {
                return Some(WidgetEvent::Dragging(id, mouse_in_px));
            }
        } else if event.state == MouseState::Down && is_inside && !data.state.is_pressed() {
            return Some(WidgetEvent::Pressed(id));
        } else if event.state == MouseState::Up && data.state.is_pressed() {
            return Some(WidgetEvent::Released(id));
        }
        None
    }

    fn manage_input(&mut self, events_rw: &mut EventsRw) {
        let data = self.get_data_mut();
        if !data.state.is_visible() || !data.state.is_active() || !data.state.is_selectable() {
            return;
        }
        let mut is_on_child = false;
        data.node.propagate_on_children(|w| {
            is_on_child |= w.get_data().state.is_hover();
        });
        if is_on_child {
            return;
        }
        let mut widget_events: Vec<WidgetEvent> = Vec::new();
        if let Some(mut mouse_events) = events_rw.read().unwrap().read_events::<MouseEvent>() {
            for event in mouse_events.iter_mut() {
                if let Some(widget_event) = self.manage_mouse_event(event) {
                    widget_events.push(widget_event);
                }
            }
        }
        for e in widget_events {
            let mut events = events_rw.write().unwrap();
            events.send_event(e);
        }
    }
    fn move_to_layer(&mut self, layer: f32) {
        if (layer - self.get_data().graphics.get_layer()).abs() > f32::EPSILON {
            let data = self.get_data_mut();
            data.graphics.move_to_layer(-data.graphics.get_layer());
            data.graphics.set_layer(layer);
            data.graphics.move_to_layer(layer);
        }
    }

    fn update_layout(&mut self) {
        self.compute_offset_and_scale_from_alignment();
        self.apply_fit_to_content();

        self.clip_in_area();
        self.update_layers();
    }

    fn update_layers(&mut self) {
        let layer = self.get_data().graphics.get_layer();
        self.get_data_mut().node.propagate_on_children_mut(|w| {
            w.move_to_layer(layer - DEFAULT_LAYER_OFFSET);
            w.update_layers();
        });
    }
    fn is_hover(&self) -> bool {
        self.get_data().state.is_hover()
    }
    fn is_hover_recursive(&self) -> bool {
        let mut is_hover = self.get_data().state.is_hover();
        if !is_hover {
            self.get_data().node.propagate_on_children(|w| {
                if w.is_hover_recursive() {
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
        id
    }
    fn remove_children(&mut self, renderer: &mut Renderer) {
        self.get_data_mut().node.propagate_on_children_mut(|w| {
            w.get_data_mut().graphics.remove_meshes(renderer);
        });
        self.get_data_mut().node.remove_children();

        self.update_layout();
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
        self.get_data_mut().state.set_visible(visible);
    }
}
